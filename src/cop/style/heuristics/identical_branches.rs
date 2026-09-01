//! Style/IdenticalConditionalBranches heuristic.

use tree_sitter::Node;

use crate::cop::CopConfig;
use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

/// Branches that share identical source text (for reporting).
pub fn identical_branch_nodes<'a>(
    source: &SourceFile,
    node: Node<'a>,
) -> Option<Vec<Node<'a>>> {
    let branches = match node.kind() {
        "if" | "unless" | "conditional" => if_branches(node),
        "case" => case_branches(node),
        _ => return None,
    };
    if branches.len() < 2 || branches.iter().any(Option::is_none) {
        return None;
    }
    let branches: Vec<Node<'a>> = branches.into_iter().flatten().collect();
    if !same_source(source, &branches) {
        return None;
    }
    Some(branches)
}

/// Kept for breadth-first Style cop wiring; prefer [`identical_branch_nodes`].
#[allow(dead_code)]
pub fn matches_identical_conditional_branches(
    source: &SourceFile,
    node: Node<'_>,
    _config: &CopConfig,
) -> bool {
    identical_branch_nodes(source, node).is_some()
}

fn if_branches<'a>(node: Node<'a>) -> Vec<Option<Node<'a>>> {
    // Ternary `conditional` uses condition/consequence/alternative fields.
    if node.kind() == "conditional" || is_ternary(node) {
        return ternary_branches(node);
    }
    let then_b = node
        .child_by_field_name("consequence")
        .or_else(|| node.child_by_field_name("body"));
    let else_b = node.child_by_field_name("alternative");
    match (then_b, else_b) {
        (Some(t), Some(e)) => then_else_branches(t, e),
        _ => vec![],
    }
}

fn then_else_branches<'a>(then_b: Node<'a>, else_b: Node<'a>) -> Vec<Option<Node<'a>>> {
    if else_b.kind() == "else" {
        let mut cur = else_b.walk();
        let body = else_b.named_children(&mut cur).next();
        return vec![Some(then_b), body];
    }
    if matches!(else_b.kind(), "if" | "elsif") {
        let mut out = vec![Some(then_b)];
        out.extend(if_branches(else_b));
        return out;
    }
    vec![Some(then_b), Some(else_b)]
}

fn ternary_branches<'a>(node: Node<'a>) -> Vec<Option<Node<'a>>> {
    let then_b = node.child_by_field_name("consequence");
    let else_b = node.child_by_field_name("alternative");
    if then_b.is_some() && else_b.is_some() {
        return vec![then_b, else_b];
    }
    let mut cur = node.walk();
    let named: Vec<_> = node.named_children(&mut cur).collect();
    if named.len() >= 3 {
        vec![Some(named[1]), Some(named[2])]
    } else {
        vec![]
    }
}

fn is_ternary(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.children(&mut cur)
        .any(|c| !c.is_named() && (c.kind() == "?" || c.kind() == ":"))
}

fn case_branches<'a>(node: Node<'a>) -> Vec<Option<Node<'a>>> {
    let mut out = Vec::new();
    let mut cur = node.walk();
    for child in node.named_children(&mut cur) {
        match child.kind() {
            "when" | "else" => {
                let mut c2 = child.walk();
                let body = child.named_children(&mut c2).last();
                out.push(body);
            }
            _ => {}
        }
    }
    out
}

fn same_source(source: &SourceFile, branches: &[Node<'_>]) -> bool {
    let Some(first) = branches.first() else {
        return false;
    };
    let first_bytes = node_bytes(source, *first);
    if first_bytes.is_empty() {
        return false;
    }
    branches
        .iter()
        .all(|b| node_bytes(source, *b) == first_bytes)
}
