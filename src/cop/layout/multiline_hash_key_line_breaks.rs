//! Layout/Layout/MultilineHashKeyLineBreaks.

use tree_sitter::Node;

use crate::cop::layout::line_breaks;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineHashKeyLineBreaks;

impl Cop for MultilineHashKeyLineBreaks {
    fn name(&self) -> &'static str { "Layout/MultilineHashKeyLineBreaks" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["hash"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        line_breaks::check_breaks(
            self, source, node, "Each key in a multi-line hash must start on a separate line.",
            diagnostics, &mut corrections,
        );
    }
}
