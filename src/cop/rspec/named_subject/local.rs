//! A `subject` read that is a Ruby local, not the RSpec helper.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

const SCOPES: &[&str] = &[
    "method",
    "singleton_method",
    "class",
    "module",
    "singleton_class",
    "block",
    "do_block",
    "lambda",
    "program",
];

const STOPS: &[&str] = &[
    "method",
    "singleton_method",
    "class",
    "module",
    "singleton_class",
    "program",
];

pub(super) fn is_local(source: &SourceFile, node: Node<'_>) -> bool {
    let mut parent = node.parent();
    while let Some(scope) = parent {
        if SCOPES.contains(&scope.kind()) && scope_binds(source, scope) {
            return true;
        }
        if STOPS.contains(&scope.kind()) {
            break;
        }
        parent = scope.parent();
    }
    false
}

fn scope_binds(source: &SourceFile, scope: Node<'_>) -> bool {
    scope
        .child_by_field_name("parameters")
        .is_some_and(|params| params_bind(source, params))
        || assigns_in(source, scope)
}

fn params_bind(source: &SourceFile, params: Node<'_>) -> bool {
    let mut cur = params.walk();
    params
        .named_children(&mut cur)
        .any(|child| param_binds(source, child))
}

fn param_binds(source: &SourceFile, node: Node<'_>) -> bool {
    if is_subject(source, node) {
        return true;
    }
    if node
        .child_by_field_name("name")
        .is_some_and(|name| is_subject(source, name))
    {
        return true;
    }
    if !matches!(node.kind(), "parameters" | "destructured_parameter") {
        return false;
    }
    params_bind(source, node)
}

fn assigns_in(source: &SourceFile, scope: Node<'_>) -> bool {
    let mut stack = vec![scope];
    while let Some(node) = stack.pop() {
        if node.id() != scope.id() && SCOPES.contains(&node.kind()) {
            continue;
        }
        if binds_here(source, node) {
            return true;
        }
        let mut cur = node.walk();
        for child in node.children(&mut cur) {
            stack.push(child);
        }
    }
    false
}

fn binds_here(source: &SourceFile, node: Node<'_>) -> bool {
    if node.kind() == "exception_variable" {
        return params_bind(source, node);
    }
    node.child_by_field_name("left")
        .or_else(|| node.child_by_field_name("pattern"))
        .is_some_and(|left| lhs_is_subject(source, left))
}

fn lhs_is_subject(source: &SourceFile, node: Node<'_>) -> bool {
    if is_subject(source, node) {
        return true;
    }
    if !matches!(
        node.kind(),
        "left_assignment_list" | "destructured_left_assignment" | "rest_assignment"
    ) {
        return false;
    }
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|child| lhs_is_subject(source, child))
}

fn is_subject(source: &SourceFile, node: Node<'_>) -> bool {
    node.kind() == "identifier" && node_bytes(source, node) == b"subject"
}
