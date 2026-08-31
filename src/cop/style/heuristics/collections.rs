//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_empty_heredoc(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "heredoc_body" { return false; }
    let mut cur = node.walk();
    let content = node.children(&mut cur).find(|ch| ch.kind() == "heredoc_content");
    content.is_some_and(|c| {
        let b = &source.as_bytes()[c.start_byte()..c.end_byte()];
        b.is_empty() || b == b"\n"
    })
}

pub fn matches_hash_syntax(_source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() != "pair" { return false; }
    let style = config.get_str("EnforcedStyle", "ruby19");
    let has_rocket = has_anon_child(node, "=>");
    let has_colon = has_anon_child(node, ":");
    match style {
        "ruby19" | "ruby19_no_mixed_keys" => has_rocket && pair_key_is_symbol(node),
        "hash_rockets" => has_colon,
        _ => false,
    }
}

fn has_anon_child(node: Node<'_>, kind: &str) -> bool {
    let mut cur = node.walk();
    node.children(&mut cur).any(|ch| !ch.is_named() && ch.kind() == kind)
}

fn pair_key_is_symbol(node: Node<'_>) -> bool {
    let mut c3 = node.walk();
    node.named_children(&mut c3)
        .next()
        .is_some_and(|k| matches!(k.kind(), "simple_symbol" | "hash_key_symbol"))
}
pub fn matches_numeric_literal_prefix(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() != "integer" { return false; }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    let style = config.get_str("EnforcedOctalStyle", "zero_with_o");
    if b.starts_with(b"0") && b.len() > 1 && b[1].is_ascii_digit() {
        return style == "zero_with_o"; // 0123 should be 0o123
    }
    false
}

pub fn matches_sample(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    // a[rand(a.size)] element_reference
    if node.kind() != "element_reference" { return false; }
    let mut cur = node.walk();
    let named: Vec<_> = node.named_children(&mut cur).collect();
    if named.len() < 2 { return false; }
    let idx = named[1];
    if idx.kind() != "call" { return false; }
    crate::cop::shared::call_method_name(source, idx) == Some(b"rand")
}

pub fn matches_symbol_array(_source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    let style = config.get_str("EnforcedStyle", "percent_i");
    match node.kind() {
        "array" => {
            if style != "percent_i" { return false; }
            let mut cur = node.walk();
            let elems: Vec<_> = node.named_children(&mut cur).collect();
            if elems.len() < 2 { return false; }
            elems.iter().all(|e| matches!(e.kind(), "simple_symbol" | "delimited_symbol"))
        }
        "symbol_array" => style == "brackets",
        _ => false,
    }
}

pub fn matches_word_array(_source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    let style = config.get_str("EnforcedStyle", "percent_w");
    match node.kind() {
        "array" => {
            if style != "percent_w" { return false; }
            let mut cur = node.walk();
            let elems: Vec<_> = node.named_children(&mut cur).collect();
            if elems.len() < 2 { return false; }
            elems.iter().all(|e| e.kind() == "string")
        }
        "string_array" => style == "brackets",
        _ => false,
    }
}

