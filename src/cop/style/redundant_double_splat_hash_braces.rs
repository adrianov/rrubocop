//! Style/RedundantDoubleSplatHashBraces — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_redundant_double_splat_hash_braces;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RedundantDoubleSplatHashBraces;

impl Cop for RedundantDoubleSplatHashBraces {
    fn name(&self) -> &'static str {
        "Style/RedundantDoubleSplatHashBraces"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["hash_splat", "call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_redundant_double_splat_hash_braces(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/RedundantDoubleSplatHashBraces offense.".to_string(),
        ));
    }
}
