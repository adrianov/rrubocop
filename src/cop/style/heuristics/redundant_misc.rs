//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
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

pub fn matches_redundant_percent_q(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "string" { return false; }
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    b.starts_with(b"%q") || b.starts_with(b"%Q")
}

