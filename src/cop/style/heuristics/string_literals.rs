//! Style/StringLiterals heuristic matchers and quote helpers.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

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

pub(crate) fn double_quotes_required(src: &[u8]) -> bool {
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
    // RuboCop StringLiteralsHelp: /" | \\[^'\\] | \#[@{$]/x
    // Note: `\\.` matches starting at the *second* backslash of `\\.` — do not
    // skip ahead by 2 on `\\` or that match is missed.
    let mut i = 0;
    while i < src.len() {
        match src[i] {
            b'"' => return true,
            b'\\' if i + 1 < src.len() && !matches!(src[i + 1], b'\'' | b'\\') => {
                return true;
            }
            b'#' if src.get(i + 1).is_some_and(|b| matches!(b, b'@' | b'$' | b'{')) => {
                return true;
            }
            _ => {}
        }
        i += 1;
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
    if !string_literal_candidate(source, node) {
        return false;
    }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    quote_mismatch(config.get_str("EnforcedStyle", "single_quotes"), b)
}

fn string_literal_candidate(source: &SourceFile, node: Node<'_>) -> bool {
    if node.kind() != "string" || inside_interpolation(node) || is_multiline_string(source, node) {
        return false;
    }
    // `'0':` / `"x y":` — tree-sitter emits a string key + `:`; RuboCop sees a symbol.
    if is_label_string_key(node) {
        return false;
    }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    !b.starts_with(b"%") && !b.starts_with(b"?") && !has_interpolation(node)
}

fn is_label_string_key(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != "pair" {
        return false;
    }
    if parent.child_by_field_name("key").map(|k| k.id()) != Some(node.id()) {
        return false;
    }
    let mut cur = parent.walk();
    parent
        .children(&mut cur)
        .any(|c| !c.is_named() && c.kind() == ":")
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
