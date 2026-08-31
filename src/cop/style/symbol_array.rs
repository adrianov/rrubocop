//! Style/SymbolArray — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_symbol_array;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SymbolArray;

impl Cop for SymbolArray {
    fn name(&self) -> &'static str {
        "Style/SymbolArray"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["array", "string_array", "symbol_array"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_symbol_array(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/SymbolArray offense.".to_string(),
        ));
    }
}
