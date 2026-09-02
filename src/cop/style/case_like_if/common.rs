use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

pub(super) fn unwrap(node: Node<'_>) -> Node<'_> {
    let mut n = node;
    while matches!(n.kind(), "begin" | "parenthesized_statements") {
        let mut cur = n.walk();
        if let Some(inner) = n.named_children(&mut cur).next() {
            n = inner;
        } else {
            break;
        }
    }
    n
}

pub(super) fn binary_op<'a>(source: &'a SourceFile, node: Node<'a>) -> Option<&'a [u8]> {
    if node.kind() != "binary" {
        return None;
    }
    let mut cur = node.walk();
    node.children(&mut cur)
        .find(|c| !c.is_named())
        .map(|c| node_bytes(source, c))
}

pub(super) fn sides(node: Node<'_>) -> Option<(Node<'_>, Node<'_>)> {
    Some((
        node.child_by_field_name("left")?,
        node.child_by_field_name("right")?,
    ))
}

pub(super) fn same_target(source: &SourceFile, a: Node<'_>, b: Node<'_>) -> bool {
    a == b
        || (a.kind() == "identifier"
            && b.kind() == "identifier"
            && node_bytes(source, a) == node_bytes(source, b))
}

pub(super) fn literal_or_const(source: &SourceFile, n: Node<'_>) -> bool {
    matches!(
        n.kind(),
        "string" | "symbol" | "simple_symbol" | "integer" | "float" | "true" | "false" | "nil"
    ) || (n.kind() == "constant" && node_bytes(source, n).iter().all(|&b| b.is_ascii_uppercase()))
}
