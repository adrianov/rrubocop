//! Layout/FirstArrayElementLineBreak.

use tree_sitter::Node;

use crate::cop::layout::first_break;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FirstArrayElementLineBreak;

impl Cop for FirstArrayElementLineBreak {
    fn name(&self) -> &'static str { "Layout/FirstArrayElementLineBreak" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["array"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        first_break::check_first_break_min(
            self, source, node, 2,
            "Add a line break before the first element of a multi-line array.",
            diagnostics, &mut corrections,
        );
    }
}
