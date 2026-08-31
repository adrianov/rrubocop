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

fn first_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cur = node.walk();
    node.named_children(&mut cur).find(|n| n.kind() != "comment")
}

fn expected_close_col(source: &SourceFile, node: Node<'_>, indent_width: usize) -> usize {
    let open_line = shared::node_line(source, node);
    let open_col = shared::node_col(source, node);
    let Some(first) = first_named_child(node) else {
        // empty multiline `()` → align `)` with `(`
        return open_col;
    };
    let first_line = shared::node_line(source, first);
    if first_line > open_line {
        // First arg on next line → `)` outdented by IndentationWidth from that indent.
        let arg_indent = shared::line_indent(source, first.start_byte());
        return arg_indent.saturating_sub(indent_width);
    }
    open_col
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
