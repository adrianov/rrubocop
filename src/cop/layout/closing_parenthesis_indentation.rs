//! Layout/ClosingParenthesisIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ClosingParenthesisIndentation;

fn is_paren_wrapped(bytes: &[u8], node: Node<'_>) -> bool {
    bytes.get(node.start_byte()) == Some(&b'(')
        && bytes.get(node.end_byte().saturating_sub(1)) == Some(&b')')
}

fn named_elements(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .filter(|n| n.kind() != "comment")
        .collect()
}

/// RuboCop `all_elements_aligned?`: every element's start column is the same
/// (for a leading hash, compare its pair keys instead).
fn all_elements_aligned(source: &SourceFile, elements: &[Node<'_>]) -> bool {
    let Some(first) = elements.first() else {
        return true;
    };
    let cols: Vec<usize> = if first.kind() == "hash" {
        let mut cur = first.walk();
        first
            .named_children(&mut cur)
            .filter(|n| n.kind() == "pair")
            .map(|n| shared::node_col(source, n))
            .collect()
    } else {
        elements
            .iter()
            .map(|n| shared::node_col(source, *n))
            .collect()
    };
    let Some(c0) = cols.first() else {
        return true;
    };
    cols.iter().all(|c| c == c0)
}

/// RuboCop `expected_column` for a hanging `)`.
fn expected_close_col(source: &SourceFile, node: Node<'_>, indent_width: usize) -> usize {
    let open_line = shared::node_line(source, node);
    let open_col = shared::node_col(source, node);
    let elements = named_elements(node);
    let Some(first) = elements.first().copied() else {
        // empty multiline `()` → align `)` with `(`
        return open_col;
    };
    let first_line = shared::node_line(source, first);
    if first_line > open_line {
        // First arg on next line → `)` outdented by IndentationWidth from that indent.
        let arg_indent = shared::line_indent(source, first.start_byte());
        return arg_indent.saturating_sub(indent_width);
    }
    if all_elements_aligned(source, &elements) {
        open_col
    } else {
        // Params not lined up — outdent `)` to the first argument's line indent.
        shared::line_indent(source, first.start_byte())
    }
}

impl Cop for ClosingParenthesisIndentation {
    fn name(&self) -> &'static str {
        "Layout/ClosingParenthesisIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["argument_list", "parenthesized_statements", "method_parameters"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if !is_paren_wrapped(source.as_bytes(), node) {
            return;
        }
        let open_line = shared::node_line(source, node);
        let close_off = node.end_byte() - 1;
        let (close_line, close_col) = source.offset_to_line_col(close_off);
        if open_line == close_line {
            return;
        }
        // Only hanging `)` (starts its line).
        if shared::line_indent(source, close_off) != close_col {
            return;
        }
        let width = config.get_usize("IndentationWidth", 2);
        let want = expected_close_col(source, node, width);
        if close_col == want {
            return;
        }
        report::fix_indent(
            self,
            source,
            close_off,
            format!("Indent `)` to column {want}."),
            diagnostics,
            &mut corrections,
            shared::line_indent(source, close_off),
            want,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        ClosingParenthesisIndentation,
        "cops/layout/closing_parenthesis_indentation"
    );
}
