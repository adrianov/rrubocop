//! Layout/MultilineMethodDefinitionBraceLayout.

use tree_sitter::Node;

use crate::cop::layout::brace_layout;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineMethodDefinitionBraceLayout;

impl Cop for MultilineMethodDefinitionBraceLayout {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodDefinitionBraceLayout"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method_parameters", "parameters"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "symmetrical");
        brace_layout::check_braces(self, source, node, b'(', style, diagnostics, &mut corrections);
    }
}
