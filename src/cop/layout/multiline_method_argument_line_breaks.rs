//! Layout/Layout/MultilineMethodArgumentLineBreaks.

use tree_sitter::Node;

use crate::cop::layout::line_breaks;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineMethodArgumentLineBreaks;

impl Cop for MultilineMethodArgumentLineBreaks {
    fn name(&self) -> &'static str { "Layout/MultilineMethodArgumentLineBreaks" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["argument_list", "command_argument_list"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        line_breaks::check_breaks(
            self, source, node, "Each argument in a multi-line method call must start on a separate line.",
            diagnostics, &mut corrections,
        );
    }
}
