//! RSpec/ExpectInLet — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct ExpectInLet;


impl Cop for ExpectInLet {
    fn name(&self) -> &'static str {
        "RSpec/ExpectInLet"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
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
