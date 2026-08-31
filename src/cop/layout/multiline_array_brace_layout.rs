//! Layout/MultilineArrayBraceLayout.

use tree_sitter::Node;

use crate::cop::layout::brace_layout;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineArrayBraceLayout;

impl Cop for MultilineArrayBraceLayout {
    fn name(&self) -> &'static str {
        "Layout/MultilineArrayBraceLayout"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["array"]
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
        brace_layout::check_braces(self, source, node, b'[', style, diagnostics, &mut corrections);
    }
}
