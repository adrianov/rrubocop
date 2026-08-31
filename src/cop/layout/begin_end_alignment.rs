//! Layout/Layout/BeginEndAlignment.

use tree_sitter::Node;

use crate::cop::layout::end_align;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BeginEndAlignment;

impl Cop for BeginEndAlignment {
    fn name(&self) -> &'static str { "Layout/BeginEndAlignment" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["begin"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        end_align::check_end(
            self, source, node, "begin",
            diagnostics, &mut corrections,
        );
    }
}
