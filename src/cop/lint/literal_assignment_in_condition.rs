use tree_sitter::Node;

use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/LiteralAssignmentInCondition — `if x = 1` style with literal RHS.
pub struct LiteralAssignmentInCondition;

fn is_literal(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "integer" | "float" | "string" | "simple_symbol" | "true" | "false" | "nil" | "array" | "hash" | "regex"
    )
}

fn unwrap_parens(mut cond: Node<'_>) -> Node<'_> {
    while cond.kind() == "parenthesized_statements" {
        let mut cur = cond.walk();
        let named: Vec<_> = cond.named_children(&mut cur).collect();
        if named.len() != 1 {
            break;
        }
        cond = named[0];
    }
    cond
}

fn literal_assign(cond: Node<'_>) -> Option<Node<'_>> {
    if cond.kind() != "assignment" {
        return None;
    }
    let right = cond.child_by_field_name("right")?;
    is_literal(right).then_some(right)
}

impl Cop for LiteralAssignmentInCondition {
    fn name(&self) -> &'static str {
        "Lint/LiteralAssignmentInCondition"
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
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(cond) = node.child_by_field_name("condition") else {
            return;
        };
        let cond = unwrap_parens(cond);
        let Some(right) = literal_assign(cond) else {
            return;
        };
        let lit = node_text(source, right);
        let (line, col) = source.offset_to_line_col(cond.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!(
                "Don't use literal assignment `= {lit}` in conditional,                  it should be `==` or non-literal operand wrapped in parentheses `(...)`."
            ),
        ));
    }
}
