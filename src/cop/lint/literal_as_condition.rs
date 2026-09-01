use tree_sitter::Node;

use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/LiteralAsCondition — literal used as condition.
pub struct LiteralAsCondition;

fn is_literal(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "integer"
            | "float"
            | "string"
            | "simple_symbol"
            | "true"
            | "false"
            | "nil"
            | "regex"
            | "string_array"
            | "symbol_array"
    ) || primitive_array(node)
}

fn primitive_array(node: Node<'_>) -> bool {
    if node.kind() != "array" {
        return false;
    }
    let mut cur = node.walk();
    let kids: Vec<_> = node.named_children(&mut cur).collect();
    !kids.is_empty()
        && kids.iter().all(|c| {
            matches!(
                c.kind(),
                "integer"
                    | "float"
                    | "string"
                    | "simple_symbol"
                    | "true"
                    | "false"
                    | "nil"
                    | "regex"
            )
        })
}

fn unwrap(mut node: Node<'_>) -> Node<'_> {
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

impl Cop for LiteralAsCondition {
    fn name(&self) -> &'static str {
        "Lint/LiteralAsCondition"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if", "unless", "while", "until", "case"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(cond) = node.child_by_field_name("condition").or_else(|| node.child_by_field_name("value")) else {
            return;
        };
        let cond = unwrap(cond);
        if !is_literal(cond) || skip_infinite_loop_literal(node, source, cond) {
            return;
        }
        let lit = node_text(source, cond);
        let (line, col) = source.offset_to_line_col(cond.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Literal `{lit}` appeared as a condition."),
        ));
    }
}

/// RuboCop skips `while true` / `until false` (Style/InfiniteLoop owns those).
fn skip_infinite_loop_literal(node: Node<'_>, source: &SourceFile, cond: Node<'_>) -> bool {
    let lit = node_text(source, cond);
    (node.kind() == "while" && lit == "true") || (node.kind() == "until" && lit == "false")
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(LiteralAsCondition, "cops/lint/literal_as_condition");
}
