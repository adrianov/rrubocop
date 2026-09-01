//! Per-node BlockAlignment check body.

use tree_sitter::Node;

use super::anchors;
use super::call_scan;
use super::opener;
use super::rescue;
use crate::cop::layout::indentation_consistency_util;
use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub(super) fn check_block(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(end_kw) = shared::end_keyword(node).or_else(|| shared::last_child_kind(node, "}"))
    else {
        return;
    };
    if skip_closer(source, node, end_kw) {
        return;
    }
    let Some(opener_node) = opener::block_opener(node) else {
        return;
    };
    if opener::brace_block_in_stabby_lambda(source, opener_node) {
        return;
    }
    let style = config.get_str("EnforcedStyleAlignWith", "either");
    let cols = alignment_cols(source, node, opener_node, end_kw, style);
    if end_aligned(&cols, shared::node_col(source, end_kw)) {
        return;
    }
    emit_offense(cop, source, end_kw, opener_node, &cols, diagnostics, corrections);
}

fn skip_closer(source: &SourceFile, node: Node<'_>, end_kw: Node<'_>) -> bool {
    !indentation_consistency_util::begins_its_line(source, end_kw.start_byte())
        || shared::node_line(source, node) == shared::node_line(source, end_kw)
}

struct AlignCols<'a> {
    style: &'a str,
    expression_col: usize,
    do_line_col: usize,
    do_line_begin_col: usize,
    call_expr_col: usize,
    opener_col: usize,
}

fn alignment_cols<'a>(
    source: &SourceFile,
    node: Node<'_>,
    opener_node: Node<'_>,
    end_kw: Node<'_>,
    style: &'a str,
) -> AlignCols<'a> {
    let anchor_off = anchors::do_line_anchor_offset(source, node, opener_node);
    AlignCols {
        style,
        expression_col: expression_col_for(source, node, end_kw, style),
        do_line_col: shared::line_indent(source, anchor_off),
        do_line_begin_col: anchors::do_line_begin_col(source, opener_node),
        call_expr_col: call_scan::call_expression_col_on_opener_line(source, opener_node),
        opener_col: shared::node_col(source, opener_node),
    }
}

fn expression_col_for(source: &SourceFile, node: Node<'_>, end_kw: Node<'_>, style: &str) -> usize {
    let mut expression_col = if style == "start_of_line" {
        anchors::expression_start_for_line(source, node)
    } else {
        anchors::expression_start_col(source, node)
    };
    if rescue::closer_follows_rescue_modifier(source, end_kw) {
        expression_col = expression_col.min(anchors::block_line_indent(source, node));
    }
    expression_col
}

fn end_aligned(cols: &AlignCols<'_>, end_col: usize) -> bool {
    match cols.style {
        "start_of_block" => end_col == cols.do_line_col,
        "start_of_line" => end_col == cols.expression_col,
        _ => {
            end_col == cols.expression_col
                || end_col == cols.do_line_col
                || end_col == cols.do_line_begin_col
                || end_col == cols.call_expr_col
        }
    }
}

fn autocorrect_col(cols: &AlignCols<'_>) -> usize {
    match cols.style {
        "start_of_block" => cols.do_line_col,
        "start_of_line" => cols.expression_col,
        _ => cols.expression_col.min(cols.do_line_col).min(cols.opener_col),
    }
}

fn emit_offense(
    cop: &dyn Cop,
    source: &SourceFile,
    end_kw: Node<'_>,
    opener_node: Node<'_>,
    cols: &AlignCols<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let open_word = if opener_node.kind() == "do" { "`do`" } else { "`{`" };
    report::fix_indent(
        cop,
        source,
        end_kw.start_byte(),
        format!("`end` is not aligned with {open_word} beginning at column {}.", cols.opener_col),
        diagnostics,
        corrections,
        shared::line_indent(source, end_kw.start_byte()),
        autocorrect_col(cols),
    );
}
