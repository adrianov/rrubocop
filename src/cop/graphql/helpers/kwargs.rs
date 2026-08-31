//! Keyword-argument helpers for GraphQL Ruby DSL cops.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, node_bytes, node_text};
use crate::parse::source::SourceFile;

use super::args::sym_text;

/// Keyword pairs from trailing hash and bare pairs in argument list.
pub fn kw_pairs<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    for arg in argument_nodes(node) {
        match arg.kind() {
            "pair" => out.push(arg),
            "hash" => {
                let mut cur = arg.walk();
                for child in arg.named_children(&mut cur) {
                    if child.kind() == "pair" {
                        out.push(child);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

pub fn pair_key_text(source: &SourceFile, pair: Node<'_>) -> Option<String> {
    let key = pair.child_by_field_name("key")?;
    sym_text(source, key).or_else(|| {
        let t = node_text(source, key);
        if key.kind() == "hash_key_symbol" || t.ends_with(':') {
            Some(t.trim_end_matches(':').to_string())
        } else {
            Some(t)
        }
    })
}

pub fn find_kwarg<'a>(source: &SourceFile, node: Node<'a>, key: &str) -> Option<Node<'a>> {
    kw_pairs(node)
        .into_iter()
        .find(|pair| pair_key_text(source, *pair).as_deref() == Some(key))
}

pub fn kwarg_value<'a>(source: &SourceFile, node: Node<'a>, key: &str) -> Option<Node<'a>> {
    find_kwarg(source, node, key)?.child_by_field_name("value")
}

pub fn kwarg_sym_value(source: &SourceFile, node: Node<'_>, key: &str) -> Option<String> {
    sym_text(source, kwarg_value(source, node, key)?)
}

pub fn has_kwarg(source: &SourceFile, node: Node<'_>, key: &str) -> bool {
    find_kwarg(source, node, key).is_some()
}

pub fn kwarg_false(source: &SourceFile, node: Node<'_>, key: &str) -> bool {
    kwarg_value(source, node, key).is_some_and(|val| node_bytes(source, val) == b"false")
}
