//! Class / module body helpers for GraphQL Ruby DSL cops.

use tree_sitter::Node;

use crate::cop::shared::{named_kids, node_text};
use crate::parse::source::SourceFile;

use super::args::bare_method;

pub fn class_body_stmts<'a>(class_node: Node<'a>) -> Vec<Node<'a>> {
    class_node
        .child_by_field_name("body")
        .map(named_kids)
        .unwrap_or_default()
}

pub fn module_body_stmts<'a>(mod_node: Node<'a>) -> Vec<Node<'a>> {
    class_body_stmts(mod_node)
}

pub fn nested_class(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(n) = p {
        if n.kind() == "class" {
            return true;
        }
        p = n.parent();
    }
    false
}

pub fn class_leaf_name(source: &SourceFile, class_node: Node<'_>) -> Option<String> {
    let name = class_node.child_by_field_name("name")?;
    match name.kind() {
        "scope_resolution" => name
            .child_by_field_name("name")
            .map(|n| node_text(source, n)),
        _ => Some(node_text(source, name)),
    }
}

pub fn find_method_def<'a>(
    class_node: Node<'a>,
    source: &SourceFile,
    method: &str,
) -> Option<Node<'a>> {
    for stmt in class_body_stmts(class_node) {
        if let Some(d) = match_method(source, stmt, method) {
            return Some(d);
        }
        if stmt.kind() == "singleton_class" {
            for inner in class_body_stmts(stmt) {
                if let Some(d) = match_method(source, inner, method) {
                    return Some(d);
                }
            }
        }
    }
    None
}

fn match_method<'a>(source: &SourceFile, stmt: Node<'a>, method: &str) -> Option<Node<'a>> {
    if matches!(stmt.kind(), "method" | "singleton_method") {
        let name = stmt.child_by_field_name("name")?;
        if node_text(source, name) == method {
            return Some(stmt);
        }
    }
    None
}

pub fn enclosing_class(node: Node<'_>) -> Option<Node<'_>> {
    let mut p = node.parent();
    while let Some(n) = p {
        if n.kind() == "class" {
            return Some(n);
        }
        p = n.parent();
    }
    None
}

pub fn consecutive_lines(a: Node<'_>, b: Node<'_>) -> bool {
    a.end_position().row + 1 == b.start_position().row
}

pub fn collect_calls_named<'a>(
    root: Node<'a>,
    source: &SourceFile,
    name: &[u8],
) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    walk_calls(root, source, name, &mut out);
    out
}

fn walk_calls<'a>(node: Node<'a>, source: &SourceFile, name: &[u8], out: &mut Vec<Node<'a>>) {
    if bare_method(source, node, name) {
        out.push(node);
    }
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        walk_calls(child, source, name, out);
    }
}

pub fn superclass_name(source: &SourceFile, class_node: Node<'_>) -> Option<String> {
    let sc = class_node.child_by_field_name("superclass")?;
    let mut cur = sc.walk();
    let inner = sc.named_children(&mut cur).next().unwrap_or(sc);
    Some(node_text(source, inner))
}
