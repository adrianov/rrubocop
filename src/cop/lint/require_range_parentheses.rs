use tree_sitter::Node;

use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/RequireRangeParentheses — complex range end/begin without parens.
pub struct RequireRangeParentheses;

fn is_complex(n: Option<Node<'_>>) -> bool {
    n.is_some_and(|n| matches!(n.kind(), "binary" | "call" | "unary" | "range"))
}

fn needs_parens(node: Node<'_>) -> bool {
    if node
        .parent()
        .is_some_and(|p| p.kind() == "parenthesized_statements")
    {
        return false;
    }
    is_complex(node.child_by_field_name("begin")) || is_complex(node.child_by_field_name("end"))
}

impl Cop for RequireRangeParentheses {
    fn name(&self) -> &'static str {
        "Lint/RequireRangeParentheses"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["range"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !needs_parens(node) {
            return;
        }
        let range = node_text(source, node);
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!(
                "Wrap the range literal `{range}` in parentheses to avoid ambiguity about precedence."
            ),
        ));
    }
}
