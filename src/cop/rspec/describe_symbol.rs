//! RSpec/DescribeSymbol — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct DescribeSymbol;


impl Cop for DescribeSymbol {
    fn name(&self) -> &'static str {
        "RSpec/DescribeSymbol"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
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
