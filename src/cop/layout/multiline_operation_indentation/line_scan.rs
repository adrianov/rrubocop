//! Byte-line keyword / assignment scanning for multiline ops.

#[derive(Clone, Copy)]
pub(super) struct KeywordContext {
    pub(super) special_indent: bool,
}

pub(super) fn line_indent_bytes(line: &[u8]) -> usize {
    line.iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count()
}

pub(super) fn last_significant_index(line: &[u8]) -> Option<usize> {
    line.iter()
        .rposition(|&b| b != b' ' && b != b'\t' && b != b'\r' && b != b'\n')
}

fn is_assignment_operator(bytes: &[u8], idx: usize) -> bool {
    if bytes.get(idx) != Some(&b'=') {
        return false;
    }
    if bytes.get(idx + 1) == Some(&b'=') {
        return false;
    }
    if bytes.get(idx + 1) == Some(&b'>') {
        return false;
    }
    !matches!(
        idx.checked_sub(1).and_then(|i| bytes.get(i)),
        Some(b'=' | b'!' | b'<' | b'>')
    )
}

pub(super) fn has_assignment_before_col(line: &[u8], col: usize) -> bool {
    let end = col.min(line.len());
    (0..end)
        .rev()
        .find(|&idx| line[idx] == b'=')
        .is_some_and(|idx| is_assignment_operator(line, idx))
}

pub(super) fn line_ends_with_assignment(line: &[u8]) -> bool {
    let mut idx = match last_significant_index(line) {
        Some(idx) => idx,
        None => return false,
    };
    if line[idx] == b'\\' {
        idx = match last_significant_index(&line[..idx]) {
            Some(idx) => idx,
            None => return false,
        };
    }
    is_assignment_operator(line, idx)
}

pub(super) fn line_ends_with_logical(line: &[u8]) -> bool {
    let Some(idx) = last_significant_index(line) else {
        return false;
    };
    let trimmed = &line[..=idx];
    trimmed.ends_with(b"&&")
        || trimmed.ends_with(b"||")
        || trimmed.ends_with(b" and")
        || trimmed.ends_with(b" or")
}

fn special() -> KeywordContext {
    KeywordContext {
        special_indent: true,
    }
}

fn modifier_keyword(before: &[u8]) -> Option<KeywordContext> {
    const NEEDLES: &[&[u8]] = &[
        b" unless ",
        b" unless(",
        b" while ",
        b" while(",
        b" until ",
        b" until(",
        b" if ",
        b" if(",
    ];
    NEEDLES
        .iter()
        .any(|n| contains_kw(before, n))
        .then(special)
}

fn contains_kw(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

fn starts_kw(before: &[u8], word: &[u8]) -> bool {
    before.starts_with(word)
}

pub(super) fn keyword_on_line(line: &[u8], expr_col: usize) -> Option<KeywordContext> {
    let start = line_indent_bytes(line);
    let end = expr_col.min(line.len());
    let before = &line[start..end];
    if let Some(ctx) = leading_keyword(before) {
        return Some(ctx);
    }
    if before.starts_with(b"return ") {
        return Some(return_keyword_ctx(before));
    }
    modifier_keyword(before)
}

fn leading_keyword(before: &[u8]) -> Option<KeywordContext> {
    if starts_kw(before, b"elsif ")
        || starts_kw(before, b"if ")
        || starts_kw(before, b"if(")
        || starts_kw(before, b"unless ")
        || starts_kw(before, b"unless(")
        || starts_kw(before, b"while ")
        || starts_kw(before, b"while(")
        || starts_kw(before, b"until ")
        || starts_kw(before, b"until(")
        || starts_kw(before, b"for ")
    {
        Some(special())
    } else {
        None
    }
}

fn return_keyword_ctx(before: &[u8]) -> KeywordContext {
    if modifier_keyword(before).is_some() {
        KeywordContext {
            special_indent: false,
        }
    } else {
        special()
    }
}
