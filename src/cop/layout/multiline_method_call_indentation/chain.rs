//! Dotted call-chain root walking for MultilineMethodCallIndentation.

use tree_sitter::Node;

use crate::cop::shared;
use crate::parse::source::SourceFile;

pub(super) fn has_dot(source: &SourceFile, node: Node<'_>) -> bool {
    if let Some(op) = node.child_by_field_name("operator") {
        let t = shared::node_bytes(source, op);
        return t == b"." || t == b"&.";
    }
    let mut cur = node.walk();
    node.children(&mut cur).any(|c| {
        !c.is_named() && matches!(shared::node_bytes(source, c), b"." | b"&.")
    })
}

/// RuboCop `left_hand_side`: walk up dotted call parents so every link in a
/// chain indents from the root expression's line.
pub(super) fn chain_root<'a>(source: &SourceFile, node: Node<'a>) -> Node<'a> {
    let mut n = node.child_by_field_name("receiver").unwrap_or(node);
    loop {
        let before = n.start_byte();
        n = walk_dotted_parents(source, n);
        if !lift_from_arg_list(source, &mut n) || n.start_byte() == before {
            break;
        }
    }
    n
}

fn walk_dotted_parents<'a>(source: &SourceFile, mut n: Node<'a>) -> Node<'a> {
    while let Some(parent) = n.parent() {
        if parent.kind() == "call" && has_dot(source, parent) {
            n = parent;
        } else {
            break;
        }
    }
    n
}

fn lift_from_arg_list<'a>(source: &SourceFile, n: &mut Node<'a>) -> bool {
    let Some(args) = n.parent().filter(|p| p.kind() == "argument_list") else {
        return false;
    };
    let Some(call) = args.parent() else {
        return false;
    };
    if call.kind() == "call" && has_dot(source, call) {
        *n = call;
        true
    } else {
        false
    }
}
