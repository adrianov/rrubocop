//! Style/OneLineConditional — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_one_line_conditional;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct OneLineConditional;

impl Cop for OneLineConditional {
    fn name(&self) -> &'static str {
        "Style/OneLineConditional"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if", "unless"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_one_line_conditional(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/OneLineConditional offense.".to_string(),
        ));
    }
}
