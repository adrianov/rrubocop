//! Style/OptionalArguments — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_optional_arguments;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct OptionalArguments;

impl Cop for OptionalArguments {
    fn name(&self) -> &'static str {
        "Style/OptionalArguments"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_optional_arguments(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/OptionalArguments offense.".to_string(),
        ));
    }
}
