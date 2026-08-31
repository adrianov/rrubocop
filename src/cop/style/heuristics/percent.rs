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

pub fn matches_string_literals(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() != "string" { return false; }
    let style = config.get_str("EnforcedStyle", "single_quotes");
    let b = &source.as_bytes()[node.start_byte()..node.end_byte()];
    // skip %q/%Q/heredoc-ish
    if b.starts_with(b"%") { return false; }
    let mut cur = node.walk();
    if node.named_children(&mut cur).any(|ch| ch.kind() == "interpolation") { return false; }
    match style {
        "single_quotes" => b.starts_with(b"\""),
        "double_quotes" => b.starts_with(b"'"),
        _ => false,
    }
}
