//! Shared first-child indentation for Layout/First*Indentation cops.

use tree_sitter::Node;

use crate::cop::shared;
use crate::cop::Cop;
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

fn report_indent(
    cop: &dyn Cop,
    source: &SourceFile,
    first: Node<'_>,
    actual: usize,
    expected: usize,
    message: String,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (l, c) = source.offset_to_line_col(first.start_byte());
    let mut diag = cop.diagnostic(source, l, c, message);
    if let Some(corr) = corrections {
        if let Some(ls) = source.line_start(l) {
            corr.push(Correction {
                start: ls,
                end: ls + actual,
                replacement: " ".repeat(expected),
                cop_name: cop.name(),
                cop_index: 0,
            });
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

/// Check that the first named child of a multiline construct is indented by `width`.
pub fn check_first(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    width: usize,
    message: String,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut cur = node.walk();
    let elems: Vec<_> = node.named_children(&mut cur).collect();
    if elems.is_empty() {
        return;
    }
    let start_line = shared::node_line(source, node);
    let first = elems[0];
    if shared::node_line(source, first) == start_line {
        return;
    }
    let expected = shared::node_col(source, node) + width;
    let actual = shared::line_indent(source, first.start_byte());
    if actual != expected {
        report_indent(
            cop,
            source,
            first,
            actual,
            expected,
            message,
            diagnostics,
            corrections,
        );
    }
}
