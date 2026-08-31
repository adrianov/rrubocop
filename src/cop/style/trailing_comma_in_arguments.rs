//! Style/TrailingCommaInArguments.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct TrailingCommaInArguments;

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
        if node.start_position().row == node.end_position().row {
            return;
        }
        let close_off = node.end_byte().saturating_sub(1);
        if source.as_bytes().get(close_off) != Some(&b')') {
            return;
        }
        let (_, close_col) = source.offset_to_line_col(close_off);
        // Hanging `)` only (RuboCop TrailingCommaInArguments).
        if crate::cop::shared::line_indent(source, close_off) != close_col {
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

fn last_item(source: &SourceFile, node: Node<'_>) -> Option<(bool, usize)> {
    let mut cur = node.walk();
    let children: Vec<_> = node.children(&mut cur).collect();
    let last = children
        .iter()
        .rev()
        .find(|c| node_bytes(source, **c) != b")")?;
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
    let (line, col) = source.offset_to_line_col(start);
    let msg = match style {
        "comma" | "consistent_comma" if !has_comma => {
            "Put a comma after the last parameter of a multiline method call."
        }
        "no_comma" if has_comma => {
            "Avoid comma after the last parameter of a multiline method call."
        }
        _ => return,
    };
    diagnostics.push(cop.diagnostic(source, line, col, msg.to_string()));
}
