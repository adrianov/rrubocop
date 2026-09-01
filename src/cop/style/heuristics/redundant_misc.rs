//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::cop::style::heuristics::percent::double_quotes_required;
use crate::parse::source::SourceFile;

pub fn matches_redundant_capital_w(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "string_array" { return false; }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    b.starts_with(b"%W")
}

pub fn matches_redundant_double_splat_hash_braces(_source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "hash_splat_argument" { return false; }
    let mut cur = node.walk();
    node.named_children(&mut cur).any(|ch| ch.kind() == "hash" && {
        let mut c2 = ch.walk();
        ch.named_children(&mut c2).next().is_none()
    })
}

pub fn matches_redundant_heredoc_delimiter_quotes(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "heredoc_beginning" { return false; }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    b.contains(&b'\'') || b.contains(&b'"')
}

pub fn matches_redundant_interpolation(_source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "string" { return false; }
    let mut cur = node.walk();
    let kids: Vec<_> = node.named_children(&mut cur).collect();
    // only one interpolation and nothing else meaningful
    let interps: Vec<_> = kids.iter().filter(|k| k.kind() == "interpolation").collect();
    if interps.len() != 1 { return false; }
    kids.iter().all(|k| k.kind() == "interpolation")
}

/// RuboCop Style/RedundantPercentQ: `%q`/`%Q` only when `'`/`"` would work equally.
pub fn matches_redundant_percent_q(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "string" {
        return false;
    }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    let is_q = b.starts_with(b"%q") && !b.starts_with(b"%Q");
    let is_cap_q = b.starts_with(b"%Q");
    if !is_q && !is_cap_q {
        return false;
    }
    // Both `'` and `"` appear in the literal → keep percent form.
    if b.contains(&b'\'') && b.contains(&b'"') {
        return false;
    }
    if is_q {
        return !acceptable_q(b);
    }
    !acceptable_capital_q(node, b)
}

fn has_interpolation_text(src: &[u8]) -> bool {
    // RuboCop STRING_INTERPOLATION_REGEXP = /#\{.+\}/
    let s = std::str::from_utf8(src).unwrap_or("");
    s.contains("#{") && s.contains('}')
}

fn escaped_non_backslash(src: &[u8]) -> bool {
    let mut i = 0;
    while i + 1 < src.len() {
        if src[i] == b'\\' {
            if src[i + 1] != b'\\' {
                return true;
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    false
}

fn acceptable_q(src: &[u8]) -> bool {
    has_interpolation_text(src) || escaped_non_backslash(src)
}

fn acceptable_capital_q(node: Node<'_>, src: &[u8]) -> bool {
    if !src.contains(&b'"') {
        return false;
    }
    if has_interpolation_text(src) {
        return true;
    }
    // Static %Q — allowed when double quotes are required (e.g. `\n` escapes).
    let mut cur = node.walk();
    let has_interp = node
        .named_children(&mut cur)
        .any(|ch| ch.kind() == "interpolation");
    !has_interp && double_quotes_required(src)
}
