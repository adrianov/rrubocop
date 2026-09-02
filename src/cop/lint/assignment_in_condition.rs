use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/AssignmentInCondition — assignment used as condition.
pub struct AssignmentInCondition;

fn unwrap_parens(mut node: Node<'_>) -> Node<'_> {
    while node.kind() == "parenthesized_statements" {
        let mut cur = node.walk();
        let named: Vec<_> = node.named_children(&mut cur).collect();
        if named.len() == 1 {
            node = named[0];
        } else {
            break;
        }
    }
    node
}

fn find_assign(node: Node<'_>) -> Option<Node<'_>> {
    let n = unwrap_parens(node);
    if matches!(n.kind(), "assignment" | "operator_assignment") {
        return Some(n);
    }
    None
}

impl Cop for AssignmentInCondition {
    fn name(&self) -> &'static str {
        "Lint/AssignmentInCondition"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if", "unless", "while", "until"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(cond) = node.child_by_field_name("condition") else {
            return;
        };
        let allow_safe = config.get_bool("AllowSafeAssignment", true);
        let unwrapped = unwrap_parens(cond);
        // Safe assignment: entire condition is parenthesized assignment
        if allow_safe
            && cond.kind() == "parenthesized_statements"
            && matches!(unwrapped.kind(), "assignment" | "operator_assignment")
        {
            return;
        }
        let Some(assign) = find_assign(cond) else {
            return;
        };
        // If condition itself is parenthesized assignment, treated as safe above.
        let msg = if matches!(node.kind(), "while" | "until") {
            "Use `==` if you meant to do a comparison or move the assignment up out of the condition."
                .to_string()
        } else {
            "Use `==` if you meant to do a comparison or wrap the expression in parentheses to indicate you meant to assign in a condition."
                .to_string()
        };
        let (line, col) = source.offset_to_line_col(assign.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, msg));
    }
}
