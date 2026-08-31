//! Style/ParallelAssignment — avoid simple parallel assignment.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ParallelAssignment;

fn lhs_elements(left: Node<'_>) -> Vec<Node<'_>> {
    match left.kind() {
        "left_assignment_list" | "destructured_left_assignment" => {
            let mut cur = left.walk();
            left.named_children(&mut cur)
                .filter(|n| n.kind() != ",")
                .collect()
        }
        _ => Vec::new(),
    }
}

fn rhs_elements(right: Node<'_>) -> Option<Vec<Node<'_>>> {
    // Only flag when RHS is an explicit array (RuboCop `allowed_rhs?`).
    match right.kind() {
        "array" => {
            let mut cur = right.walk();
            Some(
                right
                    .named_children(&mut cur)
                    .filter(|n| n.kind() != ",")
                    .collect(),
            )
        }
        "right_assignment_list" => {
            let mut cur = right.walk();
            Some(
                right
                    .named_children(&mut cur)
                    .filter(|n| n.kind() != ",")
                    .collect(),
            )
        }
        _ => None,
    }
}

fn has_splat(nodes: &[Node<'_>]) -> bool {
    nodes.iter().any(|n| {
        matches!(
            n.kind(),
            "splat_argument" | "rest_assignment" | "operator_assignment"
        ) || n.kind().contains("splat")
    })
}

impl Cop for ParallelAssignment {
    fn name(&self) -> &'static str {
        "Style/ParallelAssignment"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["assignment"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_simple_parallel(node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not use parallel assignment.".to_string(),
        ));
    }
}

fn is_simple_parallel(node: Node<'_>) -> bool {
    let Some(left) = node.child_by_field_name("left") else {
        return false;
    };
    let lhs = lhs_elements(left);
    if lhs.len() <= 1 || has_splat(&lhs) {
        return false;
    }
    let Some(right) = node.child_by_field_name("right") else {
        return false;
    };
    let right = if right.kind() == "rescue_modifier" {
        right.child_by_field_name("body").unwrap_or(right)
    } else {
        right
    };
    let Some(rhs) = rhs_elements(right) else {
        return false;
    };
    !has_splat(&rhs) && lhs.len() == rhs.len()
}
