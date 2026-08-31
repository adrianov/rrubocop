//! Style/SymbolProc — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_symbol_proc;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SymbolProc;

impl Cop for SymbolProc {
    fn name(&self) -> &'static str {
        "Style/SymbolProc"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["block", "do_block", "block_argument"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_symbol_proc(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/SymbolProc offense.".to_string(),
        ));
    }
}
