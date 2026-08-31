//! FactoryBot/CreateList — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct CreateList;


impl Cop for CreateList {
    fn name(&self) -> &'static str {
        "FactoryBot/CreateList"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &[
            "**/*_spec.rb", "**/spec/**/*", "**/test/**/*",
            "**/features/**/*", "**/factories/**/*", "**/factory.rb",
        ]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["array", "pair", "block", "call", "scope_resolution", "constant", "hash", "integer", "body_statement", "string", "symbol", "command"]
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
