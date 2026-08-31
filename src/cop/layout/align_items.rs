//! Shared alignment loop for Layout/*Alignment cops.

use tree_sitter::Node;

use crate::cop::shared;
use crate::cop::Cop;
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

fn expected_col(style: &str, fixed_col: usize, base_col: usize) -> usize {
    if style == "with_fixed_indentation" {
        fixed_col
    } else {
        base_col
    }
}

fn push_indent_fix(
    corr: &mut Vec<Correction>,
    cop_name: &'static str,
    line_start: usize,
    cur_indent: usize,
    expected: usize,
) {
    corr.push(Correction {
        start: line_start,
        end: line_start + cur_indent,
        replacement: " ".repeat(expected),
        cop_name,
        cop_index: 0,
    });
}

fn report_misaligned(
    cop: &dyn Cop,
    source: &SourceFile,
    item: Node<'_>,
    expected: usize,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (l, c) = source.offset_to_line_col(item.start_byte());
    let mut diag = cop.diagnostic(source, l, c, message.into());
    if let Some(corr) = corrections {
        if let Some(line_start) = source.line_start(l) {
            let cur_indent = shared::line_indent(source, item.start_byte());
            push_indent_fix(corr, cop.name(), line_start, cur_indent, expected);
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

fn check_one(
    cop: &dyn Cop,
    source: &SourceFile,
    item: Node<'_>,
    first_line: usize,
    style: &str,
    fixed_col: usize,
    base_col: usize,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let line = shared::node_line(source, item);
    if line == first_line {
        return;
    }
    let expected = expected_col(style, fixed_col, base_col);
    if shared::node_col(source, item) != expected {
        report_misaligned(cop, source, item, expected, message, diagnostics, corrections);
    }
}

fn collect_items<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = node.walk();
    // Tree-sitter exposes inline comments as named children; RuboCop does not
    // treat them as alignable hash/arg elements.
    node.named_children(&mut cur)
        .filter(|n| n.kind() != "comment")
        .collect()
}

fn align_cols(source: &SourceFile, node: Node<'_>, items: &[Node<'_>], width: usize) -> (usize, usize, usize) {
    let first_line = shared::node_line(source, items[0]);
    let base_col = shared::node_col(source, items[0]);
    let fixed_col = shared::line_indent(source, node.start_byte()) + width;
    (first_line, base_col, fixed_col)
}

/// Align named children of `node` that span multiple lines.
pub fn check_align(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    style: &str,
    width: usize,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let items = collect_items(node);
    if items.len() < 2 {
        return;
    }
    let first_line = shared::node_line(source, items[0]);
    if !items.iter().any(|i| shared::node_line(source, *i) != first_line) {
        return;
    }
    let (first_line, base_col, fixed_col) = align_cols(source, node, &items, width);
    for item in items.iter().skip(1) {
        check_one(
            cop, source, *item, first_line, style, fixed_col, base_col, message, diagnostics,
            corrections,
        );
    }
}
