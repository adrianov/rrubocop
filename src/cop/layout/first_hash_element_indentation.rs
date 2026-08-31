//! Layout/Layout/FirstHashElementIndentation.

use tree_sitter::Node;

use crate::cop::layout::first_indent;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FirstHashElementIndentation;

impl Cop for FirstHashElementIndentation {
    fn name(&self) -> &'static str { "Layout/FirstHashElementIndentation" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["hash"] }

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
