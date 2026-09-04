use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/AssignmentInCondition — assignment used as condition.
pub struct AssignmentInCondition;

fn is_assign_kind(kind: &str) -> bool {
    matches!(kind, "assignment" | "operator_assignment")
}

fn is_conditional_op_assign(source: &SourceFile, node: Node<'_>) -> bool {
    // `||=` / `&&=` are not flagged (RuboCop).
    if node.kind() != "operator_assignment" {
        return false;
    }
    if let Some(op) = node.child_by_field_name("operator") {
        let t = node_bytes(source, op);
        return t == b"||=" || t == b"&&=";
    }
    node_bytes(source, node)
        .windows(3)
        .any(|w| w == b"||=" || w == b"&&=")
}

fn skip_descend(kind: &str) -> bool {
    matches!(
        kind,
        "block"
            | "do_block"
            | "lambda"
            | "method"
            | "singleton_method"
            | "class"
            | "module"
            | "singleton_class"
    )
}

fn is_safe_wrapped_assign(assign: Node<'_>) -> bool {
    // RuboCop: parenthesized assignment `(x = y)` is intentional even as a
    // subexpression, e.g. `until (parent = dirname(dir)) == '.'`.
    let Some(p) = assign.parent() else {
        return false;
    };
    if p.kind() != "parenthesized_statements" {
        return false;
    }
    let mut cur = p.walk();
    let named: Vec<_> = p.named_children(&mut cur).collect();
    named.len() == 1 && named[0].id() == assign.id()
}

fn collect_assigns<'a>(
    source: &SourceFile,
    node: Node<'a>,
    allow_safe: bool,
    out: &mut Vec<Node<'a>>,
) {
    if skip_descend(node.kind()) {
        return;
    }
    if is_assign_kind(node.kind())
        && !is_conditional_op_assign(source, node)
        && !(allow_safe && is_safe_wrapped_assign(node))
    {
        out.push(node);
    }
    let mut cur = node.walk();
    for child in node.named_children(&mut cur) {
        collect_assigns(source, child, allow_safe, out);
    }
}

fn loop_kind(kind: &str) -> bool {
    matches!(
        kind,
        "while" | "until" | "while_modifier" | "until_modifier"
    )
}

fn assign_message(loop_kw: bool) -> String {
    if loop_kw {
        "Use `==` if you meant to do a comparison or move the assignment up out of the condition."
            .into()
    } else {
        "Use `==` if you meant to do a comparison or wrap the expression in parentheses to indicate you meant to assign in a condition."
            .into()
    }
}

fn assign_report_byte(source: &SourceFile, assign: Node<'_>) -> usize {
    assign
        .child_by_field_name("operator")
        .or_else(|| {
            let mut cur = assign.walk();
            assign.children(&mut cur).find(|c| {
                let t = node_bytes(source, *c);
                t == b"=" || t.ends_with(b"=")
            })
        })
        .map(|n| n.start_byte())
        .unwrap_or_else(|| assign.start_byte())
}

fn report_assigns(
    cop: &AssignmentInCondition,
    source: &SourceFile,
    node: Node<'_>,
    assigns: &[Node<'_>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let msg = assign_message(loop_kind(node.kind()));
    for &assign in assigns {
        let (line, col) = source.offset_to_line_col(assign_report_byte(source, assign));
        diagnostics.push(cop.diagnostic(source, line, col, msg.clone()));
    }
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
        &[
            "if",
            "unless",
            "while",
            "until",
            "if_modifier",
            "unless_modifier",
            "while_modifier",
            "until_modifier",
        ]
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
        let mut assigns = Vec::new();
        collect_assigns(source, cond, allow_safe, &mut assigns);
        report_assigns(self, source, node, &assigns, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(AssignmentInCondition, "cops/lint/assignment_in_condition");
}
