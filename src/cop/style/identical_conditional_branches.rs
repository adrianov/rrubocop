//! Style/IdenticalConditionalBranches — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_identical_conditional_branches;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct IdenticalConditionalBranches;

impl Cop for IdenticalConditionalBranches {
    fn name(&self) -> &'static str {
        "Style/IdenticalConditionalBranches"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if", "case", "unless"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_identical_conditional_branches(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/IdenticalConditionalBranches offense.".to_string(),
        ));
    }
}
