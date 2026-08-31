//! Style/MultilineTernaryOperator — ternary must be on one line.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineTernaryOperator;

impl Cop for MultilineTernaryOperator {
    fn name(&self) -> &'static str {
        "Style/MultilineTernaryOperator"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["conditional"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.start_position().row == node.end_position().row {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Avoid multi-line ternary operators, use `if`/`else` instead.".to_string(),
        ));
    }
}
