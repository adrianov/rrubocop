//! Description / block helpers for GraphQL Ruby DSL cops.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, named_kids};
use crate::parse::source::SourceFile;

use super::args::{bare_method, call_block, positional_args};
use super::kwargs::has_kwarg;

/// Description as 3rd positional string (field/argument DSL).
pub fn positional_description(source: &SourceFile, node: Node<'_>) -> bool {
    let pos = positional_args(node);
    if pos.len() < 3 {
        return false;
    }
    let n = pos[2];
    matches!(
        n.kind(),
        "string" | "string_array" | "heredoc_beginning" | "chained_string" | "constant"
    ) || n.kind().contains("string")
        || is_stringish(source, n)
}

fn is_stringish(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "string" | "heredoc_beginning" | "chained_string" | "interpolated_string"
    ) || (node.kind() == "call" && call_method_name(source, node) == Some(b"squish"))
}

pub fn block_stmts<'a>(block: Node<'a>) -> Vec<Node<'a>> {
    let body = block.child_by_field_name("body").unwrap_or(block);
    if body.kind() == "body_statement" || body.kind() == "block_body" {
        named_kids(body)
    } else {
        vec![body]
    }
}

pub fn description_method_in(source: &SourceFile, nodes: &[Node<'_>]) -> bool {
    nodes.iter().any(|n| {
        bare_method(source, *n, b"description")
            || (matches!(n.kind(), "call" | "command" | "command_call")
                && matches!(
                    call_method_name(source, *n),
                    Some(b"description") | Some(b"description=")
                ))
    })
}

pub fn has_description(source: &SourceFile, node: Node<'_>) -> bool {
    if positional_description(source, node) || has_kwarg(source, node, "description") {
        return true;
    }
    call_block(node).is_some_and(|b| description_method_in(source, &block_stmts(b)))
}
