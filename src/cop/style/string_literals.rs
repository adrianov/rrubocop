//! Style/StringLiterals — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_string_literals;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct StringLiterals;

impl Cop for StringLiterals {
    fn name(&self) -> &'static str {
        "Style/StringLiterals"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["string", "string_content"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_string_literals(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/StringLiterals offense.".to_string(),
        ));
    }
}
