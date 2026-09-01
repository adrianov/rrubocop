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
    // RuboCop only aligns elements that begin their line (multi-key lines OK).
    if shared::node_col(source, item) != shared::line_indent(source, item.start_byte()) {
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
    // treat them as alignable hash/arg elements. Kwsplat (`**x`) is omitted from
    // RuboCop ArgumentAlignment's `hash.pairs` expansion under fixed indentation.
    node.named_children(&mut cur)
        .filter(|n| !matches!(n.kind(), "comment" | "hash_splat_argument" | "forward_argument"))
        .collect()
}

fn is_bare_hash(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(node.kind(), "hash" | "bare_assoc_hash" | "bare_hash")
        && source
            .as_bytes()
            .get(node.start_byte())
            .is_some_and(|&b| b != b'{')
}

fn hash_pairs<'a>(hash: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = hash.walk();
    hash.named_children(&mut cur)
        .filter(|n| n.kind() == "pair")
        .collect()
}

fn trailing_pair_prefix(raw: &[Node<'_>]) -> Option<usize> {
    if raw.len() >= 2 && raw[1..].iter().all(|n| n.kind() == "pair") {
        return Some(0);
    }
    let idx = raw.iter().position(|n| n.kind() == "pair")?;
    (idx > 0 && raw[idx..].iter().all(|n| n.kind() == "pair")).then_some(idx)
}

fn fixed_indent_argument_items<'a>(source: &SourceFile, raw: Vec<Node<'a>>) -> Vec<Node<'a>> {
    if raw.len() < 2 {
        return raw;
    }
    let mut items = raw[..raw.len() - 1].to_vec();
    let last = raw[raw.len() - 1];
    if is_bare_hash(source, last) {
        items.extend(hash_pairs(last));
    } else {
        items.push(last);
    }
    items
}

fn first_argument_items<'a>(source: &SourceFile, raw: Vec<Node<'a>>) -> Vec<Node<'a>> {
    let first = raw[0];
    if is_bare_hash(source, first) {
        return hash_pairs(first);
    }
    if let Some(idx) = trailing_pair_prefix(&raw) {
        return if idx == 0 {
            vec![first]
        } else {
            raw[..idx].to_vec()
        };
    }
    raw
}

/// RuboCop `flattened_arguments` for `Layout/ArgumentAlignment`.
fn collect_argument_items<'a>(source: &SourceFile, node: Node<'a>, style: &str) -> Vec<Node<'a>> {
    let raw = collect_items(node);
    if raw.is_empty() {
        return raw;
    }
    if style == "with_fixed_indentation" {
        fixed_indent_argument_items(source, raw)
    } else {
        first_argument_items(source, raw)
    }
}

fn align_cols(
    source: &SourceFile,
    node: Node<'_>,
    items: &[Node<'_>],
    width: usize,
    style: &str,
) -> (usize, usize, usize) {
    let first_line = shared::node_line(source, items[0]);
    let base_col = shared::node_col(source, items[0]);
    let fixed_col = if style == "with_fixed_indentation" {
        // RuboCop: indent of the *selector* line + width (not the receiver line).
        selector_line_indent(source, node) + width
    } else {
        let anchor = node
            .parent()
            .filter(|p| matches!(p.kind(), "call" | "command" | "command_call"))
            .unwrap_or(node);
        shared::line_indent(source, anchor.start_byte()) + width
    };
    (first_line, base_col, fixed_col)
}

fn call_selector_indent(source: &SourceFile, call: Node<'_>, arg_list: Node<'_>) -> usize {
    if let Some(method) = call.child_by_field_name("method") {
        return shared::line_indent(source, method.start_byte());
    }
    let bytes = source.as_bytes();
    let from = call.start_byte();
    let to = arg_list.start_byte().min(bytes.len());
    if let Some(rel) = bytes[from..to].iter().rposition(|&b| b == b'.' || b == b'(') {
        return shared::line_indent(source, from + rel);
    }
    shared::line_indent(source, arg_list.start_byte())
}

/// Indentation of the line that holds the method name / `(`, matching RuboCop
/// `target_method_lineno` for ArgumentAlignment fixed indentation.
fn selector_line_indent(source: &SourceFile, arg_list: Node<'_>) -> usize {
    let Some(call) = arg_list
        .parent()
        .filter(|p| matches!(p.kind(), "call" | "command" | "command_call"))
    else {
        return shared::line_indent(source, arg_list.start_byte());
    };
    call_selector_indent(source, call, arg_list)
}

fn alignment_items<'a>(source: &SourceFile, node: Node<'a>, style: &str) -> Vec<Node<'a>> {
    if node.kind() == "argument_list" {
        collect_argument_items(source, node, style)
    } else {
        collect_items(node)
    }
}

fn items_span_lines(source: &SourceFile, items: &[Node<'_>]) -> bool {
    let first_line = shared::node_line(source, items[0]);
    items.iter().any(|i| shared::node_line(source, *i) != first_line)
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
    let items = alignment_items(source, node, style);
    if items.len() < 2 || !items_span_lines(source, &items) {
        return;
    }
    let (first_line, base_col, fixed_col) = align_cols(source, node, &items, width, style);
    for item in items.iter().skip(1) {
        check_one(
            cop, source, *item, first_line, style, fixed_col, base_col, message, diagnostics,
            corrections,
        );
    }
}
