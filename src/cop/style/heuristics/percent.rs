//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_bare_percent_literals(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() != "string" { return false; }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    let style = config.get_str("EnforcedStyle", "bare_percent");
    percent_style_mismatch(b, style)
}

fn percent_style_mismatch(b: &[u8], style: &str) -> bool {
    let is_bare = is_bare_percent(b);
    let is_q = b.starts_with(b"%q") || b.starts_with(b"%Q");
    match style {
        "bare_percent" => is_q,
        "percent_q" => is_bare,
        _ => false,
    }
}

fn is_bare_percent(b: &[u8]) -> bool {
    b.starts_with(b"%(") || b.starts_with(b"%[") || b.starts_with(b"%{") || b.starts_with(b"%<")
        || (b.len() >= 2 && b[0] == b'%' && !b[1].is_ascii_alphabetic())
}

pub fn matches_command_literal(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    let style = config.get_str("EnforcedStyle", "backticks");
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    let is_pct = b.starts_with(b"%x");
    let is_bt = b.starts_with(b"`");
    if !is_pct && !is_bt { return false; }
    match style {
        "backticks" => is_pct,
        "percent_x" => is_bt,
        "mixed" => false,
        _ => false,
    }
}

pub fn matches_percent_q_literals(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() != "string" { return false; }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    let style = config.get_str("EnforcedStyle", "lower_case_q");
    match style {
        "lower_case_q" => b.starts_with(b"%Q"),
        "upper_case_q" => b.starts_with(b"%q"),
        _ => false,
    }
}

/// RuboCop `Util#double_quotes_required?` — `'` or a non-`\\`/`"` escape needs `"…"`.
fn odd_escape_needs_double(src: &[u8], i: usize) -> (usize, bool) {
    let mut n = 0;
    while i + n < src.len() && src[i + n] == b'\\' {
        n += 1;
    }
    if n % 2 == 0 {
        return (n.max(1), false);
    }
    let next = src.get(i + n).copied().unwrap_or(0);
    (n, next != b'\\' && next != b'"')
}

fn double_quotes_required(src: &[u8]) -> bool {
    let mut i = 0;
    while i < src.len() {
        match src[i] {
            b'\'' => return true,
            b'\\' => {
                let (n, need) = odd_escape_needs_double(src, i);
                if need {
                    return true;
                }
                i += n;
            }
            _ => i += 1,
        }
    }
    false
}

fn double_quotes_preferred(src: &[u8]) -> bool {
    let mut i = 0;
    while i < src.len() {
        match src[i] {
            b'"' => return true,
            b'\\' if i + 1 < src.len() => {
                if !matches!(src[i + 1], b'\'' | b'\\') {
                    return true;
                }
                i += 2;
            }
            b'#' if src.get(i + 1).is_some_and(|b| matches!(b, b'@' | b'$' | b'{')) => {
                return true;
            }
            _ => i += 1,
        }
    }
    false
}

fn has_interpolation(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|ch| ch.kind() == "interpolation")
}

fn quote_mismatch(style: &str, b: &[u8]) -> bool {
    match style {
        "single_quotes" => b.starts_with(b"\"") && !double_quotes_required(b),
        "double_quotes" => b.starts_with(b"'") && !double_quotes_preferred(b),
        _ => false,
    }
}

fn is_multiline_string(source: &SourceFile, node: Node<'_>) -> bool {
    source.offset_to_line_col(node.start_byte()).0
        != source.offset_to_line_col(node.end_byte().saturating_sub(1)).0
}

pub fn matches_string_literals(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() != "string" || inside_interpolation(node) || is_multiline_string(source, node) {
        return false;
    }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    !b.starts_with(b"%")
        && !b.starts_with(b"?")
        && !has_interpolation(node)
        && quote_mismatch(config.get_str("EnforcedStyle", "single_quotes"), b)
}

fn inside_interpolation(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(n) = p {
        if n.kind() == "interpolation" {
            return true;
        }
        // Stay within the enclosing string/symbol/regexp literal only.
        if matches!(
            n.kind(),
            "string" | "chained_string" | "symbol" | "delimited_symbol" | "regex" | "program"
        ) {
            break;
        }
        p = n.parent();
    }
    false
}
