//! Style/RedundantHeredocDelimiterQuotes — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_redundant_heredoc_delimiter_quotes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RedundantHeredocDelimiterQuotes;

impl Cop for RedundantHeredocDelimiterQuotes {
    fn name(&self) -> &'static str {
        "Style/RedundantHeredocDelimiterQuotes"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["heredoc_beginning"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_redundant_heredoc_delimiter_quotes(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/RedundantHeredocDelimiterQuotes offense.".to_string(),
        ));
    }
}
