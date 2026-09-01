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
        let then_b = node.child_by_field_name("consequence");
        let else_b = node.child_by_field_name("alternative");
        if then_b.is_some() && else_b.is_some() {
            return vec![then_b, else_b];
        }
        let mut cur = node.walk();
        let named: Vec<_> = node.named_children(&mut cur).collect();
        if named.len() >= 3 {
            return vec![Some(named[1]), Some(named[2])];
        }
    }
    let then_b = node
        .child_by_field_name("consequence")
        .or_else(|| node.child_by_field_name("body"));
    let else_b = node.child_by_field_name("alternative");
    match (then_b, else_b) {
        (Some(t), Some(e)) if e.kind() == "else" => {
            let mut cur = e.walk();
            let body = e.named_children(&mut cur).next();
            vec![Some(t), body]
        }
        (Some(t), Some(e)) if matches!(e.kind(), "if" | "elsif") => {
            let mut out = vec![Some(t)];
            out.extend(if_branches(e));
            out
        }
        (Some(t), Some(e)) => vec![Some(t), Some(e)],
        _ => vec![],
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
