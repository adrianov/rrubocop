//! Shared first-element line-break for Layout/First*LineBreak cops.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::Cop;
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Require a line break before the first named child of a multiline construct.
pub fn check_first_break(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    check_first_break_min(cop, source, node, 1, message, diagnostics, corrections);
}

pub fn check_first_break_min(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    min_elems: usize,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut cur = node.walk();
    let elems: Vec<_> = node.named_children(&mut cur).collect();
    if elems.len() < min_elems { return; }
    let start_line = shared::node_line(source, node);
    let (end_line, _) = source.offset_to_line_col(node.end_byte().saturating_sub(1));
    if start_line == end_line { return; }
    let first = elems[0];
    if shared::node_line(source, first) != start_line { return; }
    report::report_fix(
        cop, source, first.start_byte(), message.into(),
        diagnostics, corrections,
        first.start_byte(), first.start_byte(), "\n".into(),
    );
}
