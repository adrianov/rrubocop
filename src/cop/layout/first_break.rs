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
    check_first_break_cfg(cop, source, node, 1, message, diagnostics, corrections, false);
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
    check_first_break_cfg(
        cop,
        source,
        node,
        min_elems,
        message,
        diagnostics,
        corrections,
        false,
    );
}

fn uses_parens(source: &SourceFile, node: Node<'_>, first: Node<'_>) -> bool {
    let start = node.start_byte();
    let limit = first.start_byte();
    limit >= start && source.as_bytes()[start..limit].contains(&b'(')
}

fn span_end_line(source: &SourceFile, elems: &[Node<'_>], node: Node<'_>, allow_final: bool) -> usize {
    if allow_final {
        elems
            .iter()
            .map(|e| shared::node_line(source, *e))
            .max()
            .unwrap_or_else(|| shared::node_line(source, node))
    } else {
        source.offset_to_line_col(node.end_byte().saturating_sub(1)).0
    }
}

fn non_comment_kids<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .filter(|n| n.kind() != "comment")
        .collect()
}

fn first_break_target<'a>(
    source: &SourceFile,
    node: Node<'a>,
    min_elems: usize,
    allow_multiline_final: bool,
) -> Option<Node<'a>> {
    let elems = non_comment_kids(node);
    let first = *elems.first()?;
    if elems.len() < min_elems || !uses_parens(source, node, first) {
        return None;
    }
    if !first_on_call_start_line(source, node, first) {
        return None;
    }
    if call_start_line(source, node) == span_end_line(source, &elems, node, allow_multiline_final) {
        return None;
    }
    Some(first)
}

/// RuboCop uses the send/call node's first line (includes receiver). When the
/// receiver is on a prior line (`recv\n  .with(a,`), first arg is not on the
/// call's first line → no offense.
fn call_start_line(source: &SourceFile, node: Node<'_>) -> usize {
    node.parent()
        .map(|p| shared::node_line(source, p))
        .unwrap_or_else(|| shared::node_line(source, node))
}

fn first_on_call_start_line(source: &SourceFile, node: Node<'_>, first: Node<'_>) -> bool {
    call_start_line(source, node) == shared::node_line(source, first)
}

/// Like [`check_first_break_min`], with `AllowMultilineFinalElement` support.
pub fn check_first_break_cfg(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    min_elems: usize,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
    allow_multiline_final: bool,
) {
    let Some(first) = first_break_target(source, node, min_elems, allow_multiline_final) else {
        return;
    };
    report::report_fix(
        cop,
        source,
        first.start_byte(),
        message.into(),
        diagnostics,
        corrections,
        first.start_byte(),
        first.start_byte(),
        "\n".into(),
    );
}
