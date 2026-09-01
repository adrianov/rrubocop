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
    if bytes.get(start) == Some(&b':') {
        return unicode_ident_after_colon(&bytes[start + 1..]);
    }
    // Tree-sitter may emit ERROR on leftover letters after a truncated `:Н…`.
    unicode_symbol_continuation(bytes, start)
}

fn unicode_ident_after_colon(after: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(after) else {
        return false;
    };
    let Some(first) = s.chars().next() else {
        return false;
    };
    !first.is_ascii() && (first.is_alphanumeric() || first == '_')
}

fn unicode_symbol_continuation(bytes: &[u8], start: usize) -> bool {
    let Ok(s) = std::str::from_utf8(&bytes[..start]) else {
        return false;
    };
    let mut chars = s.chars().rev();
    let Some(prev) = chars.next() else {
        return false;
    };
    if !(prev.is_alphanumeric() || prev == '_') || prev.is_ascii() {
        return false;
    }
    for c in chars {
        if c == ':' {
            return true;
        }
        if !(c.is_alphanumeric() || c == '_') {
            return false;
        }
    }
    false
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
