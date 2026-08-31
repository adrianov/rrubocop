//! Layout/FirstMethodArgumentLineBreak.

use tree_sitter::Node;

use crate::cop::layout::first_break;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FirstMethodArgumentLineBreak;

impl Cop for FirstMethodArgumentLineBreak {
    fn name(&self) -> &'static str { "Layout/FirstMethodArgumentLineBreak" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["argument_list", "command_argument_list"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        first_break::check_first_break_min(
            self, source, node, 2,
            "Add a line break after the method call opening `(` or before the first argument.",
            diagnostics, &mut corrections,
        );
    }
}
