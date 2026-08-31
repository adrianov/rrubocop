//! Layout/BlockEndNewline.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BlockEndNewline;

fn end_of_block<'a>(node: Node<'a>) -> Option<Node<'a>> {
    shared::end_keyword(node).or_else(|| shared::last_child_kind(node, "}"))
}

fn last_named<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.walk();
    node.named_children(&mut cur).last()
}

fn same_line_as_end(source: &SourceFile, last: Node<'_>, end_node: Node<'_>) -> bool {
    let (last_line, _) = source.offset_to_line_col(last.end_byte().saturating_sub(1));
    last_line == shared::node_line(source, end_node)
}

impl Cop for BlockEndNewline {
    fn name(&self) -> &'static str {
        "Layout/BlockEndNewline"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["do_block", "block"]
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
        let Some(end_node) = end_of_block(node) else {
            return;
        };
        if shared::node_line(source, node) == shared::node_line(source, end_node) {
            return;
        }
        let Some(last) = last_named(node) else {
            return;
        };
        if !same_line_as_end(source, last, end_node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(last.start_byte());
        report::report_fix(
            self,
            source,
            last.start_byte(),
            format!("Expression at {line}, {col} should be on its own line."),
            diagnostics,
            &mut corrections,
            end_node.start_byte(),
            end_node.start_byte(),
            "\n".into(),
        );
    }
}
