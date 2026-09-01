//! Scan backward from a block opener for call / assignment column.

use tree_sitter::Node;

use crate::parse::source::SourceFile;

pub(super) fn call_expression_col_on_opener_line(source: &SourceFile, opener: Node<'_>) -> usize {
    let bytes = source.as_bytes();
    let line_start = line_start_of(bytes, opener.start_byte());
    let mut pos = trim_spaces_left(bytes, line_start, opener.start_byte());
    pos = scan_call_prefix(bytes, line_start, pos);
    if pos > line_start {
        pos = skip_assignment_lhs(source, line_start, pos);
    }
    pos - line_start
}

fn line_start_of(bytes: &[u8], pos: usize) -> usize {
    let mut line_start = pos;
    while line_start > 0 && bytes[line_start - 1] != b'\n' {
        line_start -= 1;
    }
    line_start
}

fn trim_spaces_left(bytes: &[u8], line_start: usize, mut pos: usize) -> usize {
    while pos > line_start && bytes[pos - 1] == b' ' {
        pos -= 1;
    }
    pos
}

fn scan_call_prefix(bytes: &[u8], line_start: usize, mut pos: usize) -> usize {
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    while pos > line_start {
        match step_call_char(bytes, line_start, pos, paren_depth, brace_depth) {
            Some((new_pos, p, b)) => {
                pos = new_pos;
                paren_depth = p;
                brace_depth = b;
            }
            None => break,
        }
    }
    pos
}

fn step_call_char(
    bytes: &[u8],
    line_start: usize,
    pos: usize,
    paren_depth: i32,
    brace_depth: i32,
) -> Option<(usize, i32, i32)> {
    let ch = bytes[pos - 1];
    if let Some(r) = step_delim(ch, pos, paren_depth, brace_depth) {
        return r;
    }
    if paren_depth > 0 || brace_depth > 0 {
        return Some((pos - 1, paren_depth, brace_depth));
    }
    step_ident_or_scope(bytes, line_start, pos, ch, paren_depth, brace_depth)
}

fn step_delim(
    ch: u8,
    pos: usize,
    paren_depth: i32,
    brace_depth: i32,
) -> Option<Option<(usize, i32, i32)>> {
    match ch {
        b')' | b']' => Some(Some((pos - 1, paren_depth + 1, brace_depth))),
        b'}' => Some(Some((pos - 1, paren_depth, brace_depth + 1))),
        b'(' | b'[' => Some(close_paren(pos, paren_depth, brace_depth)),
        b'{' => Some(close_brace(pos, paren_depth, brace_depth)),
        _ => None,
    }
}

fn close_paren(pos: usize, paren_depth: i32, brace_depth: i32) -> Option<(usize, i32, i32)> {
    (paren_depth > 0).then_some((pos - 1, paren_depth - 1, brace_depth))
}

fn close_brace(pos: usize, paren_depth: i32, brace_depth: i32) -> Option<(usize, i32, i32)> {
    (brace_depth > 0).then_some((pos - 1, paren_depth, brace_depth - 1))
}

fn step_ident_or_scope(
    bytes: &[u8],
    line_start: usize,
    pos: usize,
    ch: u8,
    paren_depth: i32,
    brace_depth: i32,
) -> Option<(usize, i32, i32)> {
    if is_call_ident_char(ch) {
        return Some((pos - 1, paren_depth, brace_depth));
    }
    if ch == b':' && pos >= 2 + line_start && bytes[pos - 2] == b':' {
        return Some((pos - 2, paren_depth, brace_depth));
    }
    None
}

fn is_call_ident_char(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'.' | b'?' | b'!' | b'@' | b'$' | b'%')
}

fn skip_assignment_lhs(source: &SourceFile, line_start: usize, pos: usize) -> usize {
    let bytes = source.as_bytes();
    let Some(eq_pos) = find_plain_eq(bytes, line_start, pos) else {
        return pos;
    };
    let lhs_end = trim_spaces_left(bytes, line_start, eq_pos);
    let lhs_pos = scan_lhs_start(bytes, line_start, lhs_end);
    if lhs_pos < lhs_end {
        lhs_pos
    } else {
        pos
    }
}

fn find_plain_eq(bytes: &[u8], line_start: usize, pos: usize) -> Option<usize> {
    let p = trim_spaces_left(bytes, line_start, pos);
    if p <= line_start || bytes[p - 1] != b'=' {
        return None;
    }
    let eq_pos = p - 1;
    if is_compound_eq(bytes, line_start, eq_pos) {
        return None;
    }
    Some(eq_pos)
}

fn is_compound_eq(bytes: &[u8], line_start: usize, eq_pos: usize) -> bool {
    if eq_pos <= line_start {
        return false;
    }
    let prev = bytes[eq_pos - 1];
    matches!(prev, b'=' | b'!' | b'<' | b'>')
        || matches!(prev, b'+' | b'-' | b'*' | b'/' | b'%' | b'^' | b'|' | b'&' | b'<')
}

fn scan_lhs_start(bytes: &[u8], line_start: usize, mut lhs_pos: usize) -> usize {
    while lhs_pos > line_start {
        match step_lhs_char(bytes, line_start, lhs_pos) {
            Some(p) => lhs_pos = p,
            None => break,
        }
    }
    lhs_pos
}

fn step_lhs_char(bytes: &[u8], line_start: usize, lhs_pos: usize) -> Option<usize> {
    let ch = bytes[lhs_pos - 1];
    if ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'@' | b'$' | b'.' | b'[') {
        return Some(lhs_pos - 1);
    }
    if ch == b':' {
        let mut p = lhs_pos - 1;
        if p > line_start && bytes[p - 1] == b':' {
            p -= 1;
        }
        return Some(p);
    }
    if ch == b',' {
        return Some(trim_spaces_left(bytes, line_start, lhs_pos - 1));
    }
    None
}
