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

fn close_off_col(source: &SourceFile, node: Node<'_>) -> (usize, usize, usize) {
    let close_off = node.end_byte() - 1;
    let (close_line, close_col) = source.offset_to_line_col(close_off);
    (close_off, close_line, close_col)
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
        let _ = config;
        if !is_paren_wrapped(source.as_bytes(), node) {
            return;
        }
        let open_line = shared::node_line(source, node);
        let (close_off, close_line, close_col) = close_off_col(source, node);
        if open_line == close_line {
            return;
        }
        let open_col = shared::node_col(source, node);
        if close_col == open_col {
            return;
        }
        report::fix_indent(
            self,
            source,
            close_off,
            format!("Indent `)` to column {open_col}."),
            diagnostics,
            &mut corrections,
            shared::line_indent(source, close_off),
            open_col,
        );
    }
}
