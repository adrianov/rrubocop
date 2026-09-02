//! Layout/Layout/FirstHashElementLineBreak.

use tree_sitter::Node;

use crate::cop::layout::first_break;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FirstHashElementLineBreak;

impl Cop for FirstHashElementLineBreak {
    fn name(&self) -> &'static str { "Layout/FirstHashElementLineBreak" }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["hash"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        first_break::check_first_break(
            self, source, node, "Add a line break before the first element of a multi-line hash.",
            diagnostics, &mut corrections,
        );
    }
}
