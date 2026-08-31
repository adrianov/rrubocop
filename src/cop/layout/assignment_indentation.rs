//! Layout/AssignmentIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct AssignmentIndentation;

impl Cop for AssignmentIndentation {
    fn name(&self) -> &'static str { "Layout/AssignmentIndentation" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["assignment", "operator_assignment"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let width = config.get_usize("IndentationWidth", 2);
        let Some(left) = node.child_by_field_name("left") else { return; };
        let Some(right) = node.child_by_field_name("right") else { return; };
        if shared::node_line(source, right) <= shared::node_line(source, left) { return; }
        let expected = shared::line_indent(source, left.start_byte()) + width;
        let actual = shared::line_indent(source, right.start_byte());
        if actual == expected { return; }
        report::fix_indent(
            self, source, right.start_byte(),
            "Indent the first line of the right-hand-side of a multi-line assignment.".into(),
            diagnostics, &mut corrections, actual, expected,
        );
    }
}
