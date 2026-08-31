use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/ElseLayout — odd else layout (statement on else line).
pub struct ElseLayout;

impl Cop for ElseLayout {
    fn name(&self) -> &'static str {
        "Lint/ElseLayout"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["else"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Odd layout: first body stmt shares line with `else` keyword.
        let mut cur = node.walk();
        let named: Vec<_> = node.named_children(&mut cur).collect();
        let Some(first) = named.first() else {
            return;
        };
        let (else_line, _) = source.offset_to_line_col(node.start_byte());
        let (stmt_line, _) = source.offset_to_line_col(first.start_byte());
        if else_line != stmt_line {
            return;
        }
        // Need another statement on a following line (RuboCop pattern)
        if named.len() < 2 {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Odd `else` layout detected. Did you mean to use `elsif`?".to_string(),
        ));
    }
}
