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
    if elems.len() < 2 || skip_breaks(source, node, &elems, allow_multiline_final) {
        return;
    }
    scan_breaks(cop, source, &elems, message, diagnostics, corrections);
}

fn is_arg_list(node: Node<'_>) -> bool {
    matches!(node.kind(), "argument_list" | "command_argument_list")
}

fn call_or_node_line(source: &SourceFile, node: Node<'_>, fallback: usize) -> usize {
    node.parent()
        .map(|p| shared::node_line(source, p))
        .unwrap_or(fallback)
}

fn breaks_end_line(source: &SourceFile, node: Node<'_>, elems: &[Node<'_>], allow_final: bool) -> usize {
    if allow_final {
        elems
            .iter()
            .map(|e| shared::node_line(source, *e))
            .max()
            .unwrap_or_else(|| shared::node_line(source, node))
    } else {
        elem_end_line(source, node)
    }
}

fn skip_breaks(
    source: &SourceFile,
    node: Node<'_>,
    elems: &[Node<'_>],
    allow_multiline_final: bool,
) -> bool {
    let start_line = shared::node_line(source, node);
    // For method args only: RuboCop send first_line includes receiver — skip when
    // args begin after the call expression's first line (chained `.with(a,`).
    if is_arg_list(node) {
        let call_start = call_or_node_line(source, node, start_line);
        if call_start != shared::node_line(source, elems[0]) {
            return true;
        }
    }
    let end_line = breaks_end_line(source, node, elems, allow_multiline_final);
    let align_start = if is_arg_list(node) {
        call_or_node_line(source, node, start_line)
    } else {
        start_line
    };
    if align_start == end_line {
        return true;
    }
    // All keys/args on one line (even if `{` / `}` wrap) → not an offense.
    if all_elems_same_line(source, elems) {
        return true;
    }
    allow_multiline_final && all_first_lines_equal(source, elems)
}
