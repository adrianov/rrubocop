//! Keyword / assignment context for multiline operation indentation.

use tree_sitter::Node;

use crate::cop::layout::indentation_consistency_util as util;
use crate::parse::source::SourceFile;

use super::line_scan::{
    has_assignment_before_col, keyword_on_line, line_ends_with_assignment, line_ends_with_logical,
    line_indent_bytes, last_significant_index, KeywordContext,
};

#[derive(Clone, Copy)]
pub(super) struct AssignmentContext {
    pub(super) rhs_begins_line: bool,
}

fn within_node(inner: Node<'_>, outer: Node<'_>) -> bool {
    inner.start_byte() >= outer.start_byte() && inner.end_byte() <= outer.end_byte()
}

pub(super) fn keyword_context(
    source: &SourceFile,
    node: Node<'_>,
    left_line: usize,
    left_col: usize,
) -> Option<KeywordContext> {
    if let Some(ctx) = keyword_from_ancestors(node) {
        return Some(ctx);
    }
    let line = source.lines().nth(left_line.saturating_sub(1)).unwrap_or(b"");
    if let Some(ctx) = keyword_on_line(line, left_col) {
        return Some(ctx);
    }
    keyword_from_prev_line(source, left_line, line)
}

fn keyword_from_prev_line(
    source: &SourceFile,
    left_line: usize,
    line: &[u8],
) -> Option<KeywordContext> {
    if left_line <= 1 {
        return None;
    }
    let prev = source.lines().nth(left_line - 2).unwrap_or(b"");
    if last_significant_index(prev).is_some_and(|idx| prev[idx] == b'\\') {
        return keyword_on_line(prev, prev.len());
    }
    let line_indent = line_indent_bytes(line);
    let prev_indent = line_indent_bytes(prev);
    if prev_indent < line_indent && line_ends_with_logical(prev) {
        return keyword_on_line(prev, prev.len());
    }
    None
}

fn keyword_from_ancestors(node: Node<'_>) -> Option<KeywordContext> {
    let mut p = node.parent();
    while let Some(anc) = p {
        if let Some(ctx) = ancestor_keyword(node, anc) {
            return Some(ctx);
        }
        p = anc.parent();
    }
    None
}

fn ancestor_keyword(node: Node<'_>, anc: Node<'_>) -> Option<KeywordContext> {
    match anc.kind() {
        "if" | "unless" | "while" | "until" | "elsif" => {
            within_condition(node, anc).then_some(KeywordContext {
                special_indent: true,
            })
        }
        "if_modifier" | "unless_modifier" | "while_modifier" | "until_modifier" => {
            within_condition(node, anc).then_some(KeywordContext {
                special_indent: false,
            })
        }
        _ => None,
    }
}

fn is_unaligned_rhs_type(kind: &str) -> bool {
    matches!(
        kind,
        "if" | "unless" | "while" | "until" | "for" | "return" | "array" | "begin"
    )
}

fn within_condition(node: Node<'_>, anc: Node<'_>) -> bool {
    anc.child_by_field_name("condition")
        .is_some_and(|cond| within_node(node, cond))
}

pub(super) fn assignment_context(
    source: &SourceFile,
    left_line: usize,
    left_col: usize,
) -> Option<AssignmentContext> {
    let line = source.lines().nth(left_line.saturating_sub(1)).unwrap_or(b"");
    if has_assignment_before_col(line, left_col) {
        return Some(AssignmentContext {
            rhs_begins_line: false,
        });
    }
    if left_line > 1 {
        let prev = source.lines().nth(left_line - 2).unwrap_or(b"");
        if line_ends_with_assignment(prev) && left_col == line_indent_bytes(line) {
            return Some(AssignmentContext {
                rhs_begins_line: true,
            });
        }
    }
    None
}

pub(super) fn assignment_from_ancestors(
    source: &SourceFile,
    node: Node<'_>,
) -> Option<AssignmentContext> {
    let mut p = node.parent();
    let mut block_disqualifies = false;
    while let Some(anc) = p {
        block_disqualifies |= ancestor_disqualifies(node, anc);
        if let Some(ctx) = assignment_at_ancestor(source, node, anc, block_disqualifies) {
            return ctx;
        }
        if matches!(
            anc.kind(),
            "program" | "method" | "singleton_method" | "class" | "module"
        ) {
            break;
        }
        p = anc.parent();
    }
    None
}

fn ancestor_disqualifies(node: Node<'_>, anc: Node<'_>) -> bool {
    matches!(anc.kind(), "do_block" | "block" | "block_body" | "lambda")
        || (is_unaligned_rhs_type(anc.kind()) && !within_condition(node, anc))
}

fn assignment_at_ancestor(
    source: &SourceFile,
    node: Node<'_>,
    anc: Node<'_>,
    block_disqualifies: bool,
) -> Option<Option<AssignmentContext>> {
    if !matches!(anc.kind(), "assignment" | "operator_assignment") {
        return None;
    }
    if block_disqualifies {
        return Some(None);
    }
    let rhs = anc.child_by_field_name("right")?;
    if within_node(node, rhs) {
        Some(Some(AssignmentContext {
            rhs_begins_line: util::begins_its_line(source, rhs.start_byte()),
        }))
    } else {
        Some(None)
    }
}
