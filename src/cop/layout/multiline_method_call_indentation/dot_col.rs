//! Dot-column helpers for MultilineMethodCallIndentation.

use tree_sitter::Node;

use crate::cop::shared;
use crate::parse::source::SourceFile;

use super::chain::has_dot;

fn call_dot_line_col(source: &SourceFile, call: Node<'_>) -> Option<(usize, usize)> {
    if !has_dot(source, call) {
        return None;
    }
    let method = call.child_by_field_name("method")?;
    let bytes = source.as_bytes();
    let from = call.start_byte();
    let to = method.start_byte().min(bytes.len());
    let rel = bytes[from..to]
        .iter()
        .rposition(|&b| b == b'.' || b == b'&')?;
    Some(source.offset_to_line_col(from + rel))
}

fn is_comment_line(line: &[u8]) -> bool {
    line.iter()
        .skip_while(|b| **b == b' ' || **b == b'\t')
        .next()
        == Some(&b'#')
}

fn dot_at_col(line: &[u8], col: usize) -> bool {
    line.get(col) == Some(&b'.')
        || (col > 0 && line.get(col - 1) == Some(&b'&') && line.get(col) == Some(&b'.'))
}

/// RuboCop `get_dot_right_above`: dot on the previous code line at the same column.
pub(super) fn dot_aligned_above(source: &SourceFile, call: Node<'_>) -> Option<usize> {
    let (line, col) = call_dot_line_col(source, call)?;
    let mut prev_line = line.saturating_sub(1);
    while prev_line >= 1 {
        let prev = source
            .line_text(prev_line)
            .map(|s| s.as_bytes())
            .unwrap_or(b"");
        if is_comment_line(prev) {
            prev_line -= 1;
            continue;
        }
        return dot_at_col(prev, col).then_some(col);
    }
    None
}

/// RuboCop `first_call_has_a_dot`: walk receivers to the first same-line call.
pub(super) fn first_same_line_dot_col(source: &SourceFile, call: Node<'_>) -> Option<usize> {
    let mut n = call;
    loop {
        if !has_dot(source, n) {
            return None;
        }
        let recv = n.child_by_field_name("receiver")?;
        let method = n.child_by_field_name("method")?;
        if shared::node_line(source, recv) == shared::node_line(source, method) {
            return call_dot_line_col(source, n).map(|(_, c)| c);
        }
        if recv.kind() == "call" {
            n = recv;
            continue;
        }
        return None;
    }
}
