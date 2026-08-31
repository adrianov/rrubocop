//! Style/TrailingCommaInArguments.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct TrailingCommaInArguments;

fn hanging_paren_list(source: &SourceFile, node: Node<'_>) -> Option<usize> {
    if node.start_position().row == node.end_position().row {
        return None;
    }
    let bytes = source.as_bytes();
    if bytes.get(node.start_byte()) != Some(&b'(') {
        return None;
    }
    let close = node.end_byte().saturating_sub(1);
    if bytes.get(close) != Some(&b')') {
        return None;
    }
    let (_, close_col) = source.offset_to_line_col(close);
    (crate::cop::shared::line_indent(source, close) == close_col).then_some(close)
}

fn last_item(source: &SourceFile, node: Node<'_>) -> Option<(bool, usize)> {
    let mut cur = node.walk();
    let children: Vec<_> = node.children(&mut cur).collect();
    let last = children.iter().rev().find(|c| {
        node_bytes(source, **c) != b")" && c.kind() != "comment"
    })?;
    Some((node_bytes(source, *last) == b",", last.start_byte()))
}

fn report(
    cop: &TrailingCommaInArguments,
    source: &SourceFile,
    style: &str,
    has_comma: bool,
    start: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let msg = match style {
        "comma" | "consistent_comma" if !has_comma => {
            "Put a comma after the last parameter of a multiline method call."
        }
        "no_comma" if has_comma => {
            "Avoid comma after the last parameter of a multiline method call."
        }
        _ => return,
    };
    let (line, col) = source.offset_to_line_col(start);
    diagnostics.push(cop.diagnostic(source, line, col, msg.to_string()));
}

impl Cop for TrailingCommaInArguments {
    fn name(&self) -> &'static str {
        "Style/TrailingCommaInArguments"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["argument_list"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if hanging_paren_list(source, node).is_none() {
            return;
        }
        let Some((has_comma, start)) = last_item(source, node) else {
            return;
        };
        report(
            self,
            source,
            config.get_str("EnforcedStyleForMultiline", "no_comma"),
            has_comma,
            start,
            diagnostics,
        );
    }
}
