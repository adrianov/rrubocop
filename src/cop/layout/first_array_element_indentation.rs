//! Layout/Layout/FirstArrayElementIndentation.

use tree_sitter::Node;

use crate::cop::layout::first_indent;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FirstArrayElementIndentation;

impl Cop for FirstArrayElementIndentation {
    fn name(&self) -> &'static str { "Layout/FirstArrayElementIndentation" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["array"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let width = config.get_usize("IndentationWidth", 2);
        let style = config.get_str("EnforcedStyle", "special_inside_parentheses");
        first_indent::check_first(
            self, source, node, width, style,
            format!("Use {width} spaces for indentation of the first element."),
            diagnostics, &mut corrections,
        );
    }
}
