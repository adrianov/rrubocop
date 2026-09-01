//! Tree-sitter ERROR shapes that MRI accepts — suppress Lint/Syntax for these.

mod anonymous;
mod pattern;

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

use self::anonymous::anonymous_block_arg_error;
use self::pattern::pattern_match_arrow_error;

/// Suppress tree-sitter ERRORs for MRI-valid Unicode symbols, endless bodies, etc.
pub(super) fn mri_valid_false_error(source: &SourceFile, node: Node<'_>) -> bool {
    unicode_symbol_error(source, node)
        || endless_method_rhs_error(source, node)
        || anonymous_block_arg_error(source, node)
        || pattern_match_arrow_error(source, node)
}

/// Endless method: `def name(...) = expr` (Ruby 3.0+). Returns byte offset of `=`.
pub(super) fn endless_eq_offset(source: &SourceFile, node: Node<'_>) -> Option<usize> {
    if !matches!(node.kind(), "method" | "singleton_method") {
        return None;
    }
    let mut has_end = false;
    let mut eq_off = None;
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        match child.kind() {
            "end" => has_end = true,
            "=" if node_bytes(source, child) == b"=" => eq_off = Some(child.start_byte()),
            _ => {}
        }
    }
    if has_end {
        None
    } else {
        eq_off
    }
}

fn unicode_symbol_error(source: &SourceFile, node: Node<'_>) -> bool {
    let bytes = source.as_bytes();
    let start = node.start_byte();
    if bytes.get(start) != Some(&b':') {
        return false;
    }
    let Ok(s) = std::str::from_utf8(&bytes[start + 1..]) else {
        return false;
    };
    let Some(first) = s.chars().next() else {
        return false;
    };
    // ASCII symbols parse fine; non-ASCII identifier start is truncated by tree-sitter.
    !first.is_ascii() && (first.is_alphanumeric() || first == '_')
}

/// Endless method RHS that tree-sitter misparses (`= raise`, `= logger.info '…'`).
///
/// Only suppress ERROR nodes that are **direct children** of an endless
/// `method`/`singleton_method` and sit after that method's `=`.
fn endless_method_rhs_error(source: &SourceFile, node: Node<'_>) -> bool {
    if !node.is_error() {
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    let Some(eq_off) = endless_eq_offset(source, parent) else {
        return false;
    };
    node.start_byte() > eq_off && node.start_byte() < parent.end_byte()
}
