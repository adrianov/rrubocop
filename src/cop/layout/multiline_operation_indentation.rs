//! Layout/MultilineOperationIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineOperationIndentation;

fn expected_col(source: &SourceFile, left: tree_sitter::Node<'_>, style: &str, width: usize) -> usize {
    if style == "indented" {
        shared::line_indent(source, left.start_byte()) + width
    } else {
        shared::node_col(source, left)
    }
}

impl Cop for MultilineOperationIndentation {
    fn name(&self) -> &'static str { "Layout/MultilineOperationIndentation" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["binary"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let width = config.get_usize("IndentationWidth", 2);
        let style = config.get_str("EnforcedStyle", "aligned");
        let Some(left) = node.child_by_field_name("left") else { return; };
        let Some(right) = node.child_by_field_name("right") else { return; };
        if shared::node_line(source, left) == shared::node_line(source, right) { return; }
        let expected = expected_col(source, left, style, width);
        let actual = shared::line_indent(source, right.start_byte());
        if actual == expected { return; }
        report::fix_indent(
            self, source, right.start_byte(),
            format!("Align operands in a multi-line operation (expected column {expected})."),
            diagnostics, &mut corrections, actual, expected,
        );
    }
}
