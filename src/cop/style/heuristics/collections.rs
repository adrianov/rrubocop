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

pub fn matches_hash_syntax(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() != "pair" {
        return false;
    }
    let style = config.get_str("EnforcedStyle", "ruby19");
    let has_rocket = has_anon_child(node, "=>");
    let has_colon = has_anon_child(node, ":");
    match style {
        // RuboCop ruby19: only rewrite when *every* key is a word symbol.
        "ruby19" | "ruby19_no_mixed_keys" => {
            has_rocket && pair_key_is_symbol(source, node) && hash_all_word_symbol_keys(source, node)
        }
        "hash_rockets" => has_colon,
        _ => false,
    }
}

fn hash_all_word_symbol_keys(source: &SourceFile, pair: Node<'_>) -> bool {
    let pairs = sibling_pairs(pair);
    !pairs.is_empty() && pairs.iter().all(|p| pair_key_is_symbol(source, *p))
}

fn sibling_pairs(pair: Node<'_>) -> Vec<Node<'_>> {
    let Some(parent) = pair.parent() else {
        return vec![pair];
    };
    if !matches!(parent.kind(), "hash" | "argument_list") {
        return vec![pair];
    }
    let mut cur = parent.walk();
    parent
        .named_children(&mut cur)
        .filter(|n| n.kind() == "pair")
        .collect()
}

fn has_anon_child(node: Node<'_>, kind: &str) -> bool {
    let mut cur = node.walk();
    node.children(&mut cur).any(|ch| !ch.is_named() && ch.kind() == kind)
}

fn pair_key_is_symbol(source: &SourceFile, node: Node<'_>) -> bool {
    let mut c3 = node.walk();
    let Some(key) = node.named_children(&mut c3).next() else {
        return false;
    };
    if !matches!(key.kind(), "simple_symbol" | "hash_key_symbol") {
        return false;
    }
    // RuboCop: setter symbols (`:foo=`) cannot use Ruby 1.9 label syntax.
    let bytes = &source.as_bytes()[key.start_byte()..key.end_byte()];
    !bytes.ends_with(b"=")
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
