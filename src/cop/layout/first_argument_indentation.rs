//! Layout/Layout/FirstArgumentIndentation.

use tree_sitter::Node;

use crate::cop::layout::first_indent;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FirstArgumentIndentation;

impl Cop for FirstArgumentIndentation {
    fn name(&self) -> &'static str { "Layout/FirstArgumentIndentation" }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["argument_list"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if !first_indent::argument_list_opens_with_paren(node) {
            return;
        }
        let width = config.get_usize("IndentationWidth", 2);
        let style = config.get_str("EnforcedStyle", "special_for_inner_method_call_in_parentheses");
        first_indent::check_first(
            self, source, node, width, style,
            format!("Use {width} spaces for indentation of the first argument."),
            diagnostics, &mut corrections,
        );
    }
}
