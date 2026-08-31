//! Rails/IndexBy — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct IndexBy;


impl Cop for IndexBy {
    fn name(&self) -> &'static str {
        "Rails/IndexBy"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["array", "block", "call", "hash", "identifier", "body_statement", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Breadth-first stub: not implemented — avoid method-name false positives.
        let _ = (source, node, diagnostics);
    }
}
