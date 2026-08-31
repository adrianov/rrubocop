//! Shared per-item line-break checks for Layout/Multiline*LineBreaks cops.

use tree_sitter::Node;

use crate::cop::shared;
use crate::cop::Cop;
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

fn report_same_line(
    cop: &dyn Cop,
    source: &SourceFile,
    item: Node<'_>,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(item.start_byte());
    let mut diag = cop.diagnostic(source, line, col, message.into());
    if let Some(corr) = corrections {
        corr.push(Correction {
            start: item.start_byte(),
            end: item.start_byte(),
            replacement: "\n".into(),
            cop_name: cop.name(),
            cop_index: 0,
        });
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn elem_end_line(source: &SourceFile, n: Node<'_>) -> usize {
    source.offset_to_line_col(n.end_byte().saturating_sub(1)).0
}

fn named_elems<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .filter(|n| n.kind() != "comment")
        .collect()
}

fn all_first_lines_equal(source: &SourceFile, elems: &[Node<'_>]) -> bool {
    let first = shared::node_line(source, elems[0]);
    elems
        .iter()
        .all(|e| shared::node_line(source, *e) == first)
}

/// RuboCop `MultilineElementLineBreaks#all_on_same_line?` (without `ignore_last`):
/// braces may wrap lines while every element still sits on one line.
fn all_elems_same_line(source: &SourceFile, elems: &[Node<'_>]) -> bool {
    let first = shared::node_line(source, elems[0]);
    let last_end = elem_end_line(source, *elems.last().unwrap());
    first == last_end
}

fn scan_breaks(
    cop: &dyn Cop,
    source: &SourceFile,
    elems: &[Node<'_>],
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut last_seen = 0usize;
    let mut seen = false;
    for e in elems {
        let first = shared::node_line(source, *e);
        if seen && last_seen >= first {
            report_same_line(cop, source, *e, message, diagnostics, corrections);
        } else {
            last_seen = elem_end_line(source, *e);
            seen = true;
        }
    }
}

/// Require each named child on its own line when the parent spans lines.
pub fn check_breaks(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    check_breaks_cfg(cop, source, node, message, diagnostics, corrections, false);
}

/// When `allow_multiline_final`, RuboCop `AllowMultilineFinalElement`: if every
/// element's *first* line equals the first element's, do not flag
/// (`foo(a, b, { … })`).
pub fn check_breaks_cfg(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
    allow_multiline_final: bool,
) {
    let elems = named_elems(node);
    if elems.len() < 2 {
        return;
    }
    let start_line = shared::node_line(source, node);
    let end_line = elem_end_line(source, node);
    if start_line == end_line {
        return;
    }
    // All keys/args on one line (even if `{` / `}` wrap) → not an offense.
    if all_elems_same_line(source, &elems) {
        return;
    }
    if allow_multiline_final && all_first_lines_equal(source, &elems) {
        return;
    }
    scan_breaks(cop, source, &elems, message, diagnostics, corrections);
}
