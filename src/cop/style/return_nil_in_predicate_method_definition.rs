//! Style/ReturnNilInPredicateMethodDefinition — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_return_nil_in_predicate_method_definition;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ReturnNilInPredicateMethodDefinition;

impl Cop for ReturnNilInPredicateMethodDefinition {
    fn name(&self) -> &'static str {
        "Style/ReturnNilInPredicateMethodDefinition"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["return", "method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_return_nil_in_predicate_method_definition(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/ReturnNilInPredicateMethodDefinition offense.".to_string(),
        ));
    }
}
