//! Layout/Layout/MultilineArrayLineBreaks.

use tree_sitter::Node;

use crate::cop::layout::line_breaks;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineArrayLineBreaks;

impl Cop for MultilineArrayLineBreaks {
    fn name(&self) -> &'static str { "Layout/MultilineArrayLineBreaks" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["array"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        line_breaks::check_breaks(
            self, source, node, "Each item in a multi-line array must start on a separate line.",
            diagnostics, &mut corrections,
        );
    }
}
