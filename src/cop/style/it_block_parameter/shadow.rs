//! Locals that shadow Ruby 3.4's implicit `it` parameter.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

fn scope_boundary(kind: &str) -> bool {
    matches!(
        kind,
        "method"
            | "singleton_method"
            | "class"
            | "module"
            | "singleton_class"
            | "block"
            | "do_block"
            | "lambda"
    )
}

fn enter(node: Node<'_>, root: Node<'_>, skip: Option<Node<'_>>) -> bool {
    if node.id() == root.id() {
        return true;
    }
    if skip.is_some_and(|s| s.id() == node.id()) {
        return false;
    }
    !scope_boundary(node.kind())
}

pub(super) fn walk_skip<'a>(root: Node<'a>, skip: Option<Node<'a>>, mut f: impl FnMut(Node<'a>)) {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if !enter(n, root, skip) {
            continue;
        }
        f(n);
        let mut cur = n.walk();
        for child in n.children(&mut cur) {
            stack.push(child);
        }
    }
}

fn assigns_name(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    if !matches!(
        node.kind(),
        "assignment" | "command_assignment" | "operator_assignment"
    ) {
        return false;
    }
    node.child_by_field_name("left")
        .is_some_and(|left| left.kind() == "identifier" && node_bytes(source, left) == name)
}

fn any_assign(source: &SourceFile, root: Node<'_>, skip: Option<Node<'_>>) -> bool {
    let mut found = false;
    walk_skip(root, skip, |n| {
        found = found || assigns_name(source, n, b"it");
    });
    found
}

pub(super) fn body_assigns_it(source: &SourceFile, body: Node<'_>) -> bool {
    any_assign(source, body, None)
}

pub(super) fn scope_assigns_it(source: &SourceFile, scope: Node<'_>, block: Node<'_>) -> bool {
    any_assign(source, scope, Some(block))
}

pub(super) fn enclosing_scope<'a>(block: Node<'a>) -> Node<'a> {
    let mut cur = block.parent();
    while let Some(n) = cur {
        if matches!(
            n.kind(),
            "method" | "singleton_method" | "class" | "module" | "singleton_class" | "program"
        ) {
            return n;
        }
        cur = n.parent();
    }
    block
}

fn param_tree_has(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    if node.kind() == "identifier" && node_bytes(source, node) == name {
        return true;
    }
    if matches!(node.kind(), "optional_parameter" | "keyword_parameter") {
        return node
            .child_by_field_name("name")
            .is_some_and(|n| node_bytes(source, n) == name);
    }
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|c| param_tree_has(source, c, name))
}

fn stops_scope(kind: &str) -> bool {
    matches!(
        kind,
        "method" | "singleton_method" | "class" | "module" | "singleton_class"
    )
}

pub(super) fn outer_param(source: &SourceFile, block: Node<'_>) -> bool {
    let mut cur = block.parent();
    while let Some(n) = cur {
        if let Some(params) = n.child_by_field_name("parameters")
            && param_tree_has(source, params, b"it")
        {
            return true;
        }
        if stops_scope(n.kind()) {
            break;
        }
        cur = n.parent();
    }
    false
}
