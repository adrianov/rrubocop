//! Layout/Layout/FirstParameterIndentation.

use tree_sitter::Node;

use crate::cop::layout::first_indent;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FirstParameterIndentation;

impl Cop for FirstParameterIndentation {
    fn name(&self) -> &'static str { "Layout/FirstParameterIndentation" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["method_parameters", "parameters"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let width = config.get_usize("IndentationWidth", 2);
        let _ = config.get_str("EnforcedStyle", "special_inside_parentheses");
        first_indent::check_first(
            self, source, node, width,
            format!("Use {width} spaces for indentation of the first parameter."),
            diagnostics, &mut corrections,
        );
    }
}
