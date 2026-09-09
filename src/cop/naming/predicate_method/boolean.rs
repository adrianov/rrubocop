//! Boolean / literal classification for Naming/PredicateMethod.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, node_bytes};
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

fn wayward(name: &[u8], config: &CopConfig) -> bool {
    match config.options.get("WaywardPredicates") {
        Some(serde_yml::Value::Sequence(items)) => items
            .iter()
            .any(|v| v.as_str().is_some_and(|s| s.as_bytes() == name)),
        _ => matches!(name, b"nonzero?" | b"infinite?"),
    }
}

fn comparison_op(op: &[u8]) -> bool {
    matches!(
        op,
        b"==" | b"===" | b"!=" | b"=~" | b"!~" | b">" | b">=" | b"<" | b"<=" | b"<=>"
    )
}

fn binary_is_comparison(source: &SourceFile, node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.children(&mut cur)
        .any(|c| !c.is_named() && comparison_op(node_bytes(source, c)))
}

fn unary_is_bang(source: &SourceFile, node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.children(&mut cur)
        .any(|c| !c.is_named() && node_bytes(source, c) == b"!")
}

/// RuboCop Parser `:block` (call + `do`/`{}`) is not `call_type?` — opaque.
fn has_block_body(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|c| c.kind() == "block" || c.kind() == "do_block")
}

fn call_returning_boolean(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if has_block_body(node) {
        return false;
    }
    let Some(name) = call_method_name(source, node) else {
        return false;
    };
    !wayward(name, config) && (name.ends_with(b"?") || comparison_op(name))
}

fn method_returning_boolean(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    match node.kind() {
        "binary" => binary_is_comparison(source, node),
        "unary" => unary_is_bang(source, node),
        "call" | "command" | "command_call" => call_returning_boolean(source, node, config),
        _ => false,
    }
}

pub(super) fn is_non_boolean_literal(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "integer"
            | "float"
            | "complex"
            | "rational"
            | "string"
            | "string_array"
            | "symbol_array"
            | "simple_symbol"
            | "symbol"
            | "delimited_symbol"
            | "hash"
            | "array"
            | "regex"
            | "nil"
    )
}

pub(super) fn boolean_return(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    matches!(node.kind(), "true" | "false") || method_returning_boolean(source, node, config)
}

pub(super) fn unknown_call(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    // Call+block is Opaque in RuboCop (not call_type?), so not "unknown" either.
    matches!(node.kind(), "call" | "command" | "command_call")
        && !has_block_body(node)
        && !method_returning_boolean(source, node, config)
}
