//! Layout/AssignmentIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct AssignmentIndentation;

fn assignment_eq_line(
    source: &SourceFile,
    node: Node<'_>,
    left: Node<'_>,
    right: Node<'_>,
) -> usize {
    let search_from = left.end_byte();
    let search_to = right.start_byte();
    let bytes = source.as_bytes();
    if let Some(rel) = bytes[search_from..search_to].iter().position(|&b| b == b'=') {
        return source.offset_to_line_col(search_from + rel).0;
    }
    shared::node_line(source, node)
}

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
        // RuboCop keys off the `=` line — mass-assign `a,\n b = x` keeps `=` with last LHS.
        let eq_line = assignment_eq_line(source, node, left, right);
        if shared::node_line(source, right) <= eq_line {
            return;
        }
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
