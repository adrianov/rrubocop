//! Collect `self.foo` assignment names in an enclosing type.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver};
use crate::parse::source::SourceFile;

pub(super) fn self_assign_names_in_enclosing_type(
    source: &SourceFile,
    node: Node<'_>,
) -> Vec<Vec<u8>> {
    let Some(root) = enclosing_type(node) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    collect_self_assign_names(source, root, &mut names);
    names
}

fn enclosing_type(node: Node<'_>) -> Option<Node<'_>> {
    let mut p = node.parent();
    while let Some(n) = p {
        if matches!(n.kind(), "class" | "module" | "singleton_class") {
            return Some(n);
        }
        p = n.parent();
    }
    None
}

fn collect_self_assign_names(source: &SourceFile, node: Node<'_>, out: &mut Vec<Vec<u8>>) {
    if matches!(node.kind(), "assignment" | "operator_assignment") {
        if let Some(left) = node.child_by_field_name("left") {
            push_self_call_name(source, left, out);
        }
    }
    let mut cur = node.walk();
    for c in node.children(&mut cur) {
        collect_self_assign_names(source, c, out);
    }
}

fn push_self_call_name(source: &SourceFile, node: Node<'_>, out: &mut Vec<Vec<u8>>) {
    let node = unwrap_index_recv(node);
    if node.kind() != "call" {
        return;
    }
    if call_receiver(node).is_none_or(|r| r.kind() != "self") {
        return;
    }
    if let Some(method) = call_method_name(source, node) {
        let bare = method.strip_suffix(b"=").unwrap_or(method);
        if !out.iter().any(|n| n == bare) {
            out.push(bare.to_vec());
        }
    }
}

fn unwrap_index_recv(mut node: Node<'_>) -> Node<'_> {
    for _ in 0..4 {
        if node.kind() != "element_reference" {
            break;
        }
        if let Some(obj) = node
            .child_by_field_name("object")
            .or_else(|| node.child_by_field_name("receiver"))
        {
            node = obj;
            continue;
        }
        break;
    }
    node
}
