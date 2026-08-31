//! Layout/MultilineOperationIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineOperationIndentation;

fn expected_col(source: &SourceFile, left: Node<'_>, style: &str, width: usize) -> usize {
    if style == "indented" {
        shared::line_indent(source, left.start_byte()) + width
    } else {
        shared::node_col(source, left)
    }
}

/// RuboCop skips ops inside `(...)` groups and parenthesized argument lists.
fn not_for_this_cop(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(n) = p {
        if n.kind() == "parenthesized_statements" {
            return true;
        }
        if n.kind() == "argument_list" {
            let mut cur = n.walk();
            if n.children(&mut cur).any(|c| !c.is_named() && c.kind() == "(") {
                return true;
            }
        }
        p = n.parent();
    }
    false
}

impl Cop for MultilineOperationIndentation {
    fn name(&self) -> &'static str {
        "Layout/MultilineOperationIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["binary"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if not_for_this_cop(node) {
            return;
        }
        let Some((actual, expected)) = indent_mismatch(source, node, config) else {
            return;
        };
        let right = node.child_by_field_name("right").unwrap();
        report::fix_indent(
            self,
            source,
            right.start_byte(),
            format!("Align operands in a multi-line operation (expected column {expected})."),
            diagnostics,
            &mut corrections,
            actual,
            expected,
        );
    }
}

fn indent_mismatch(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> Option<(usize, usize)> {
    let width = config.get_usize("IndentationWidth", 2);
    let style = config.get_str("EnforcedStyle", "aligned");
    let left = node.child_by_field_name("left")?;
    let right = node.child_by_field_name("right")?;
    if shared::node_line(source, left) == shared::node_line(source, right) {
        return None;
    }
    if shared::node_col(source, right) != shared::line_indent(source, right.start_byte()) {
        return None;
    }
    let expected = expected_col(source, left, style, width);
    let actual = shared::line_indent(source, right.start_byte());
    (actual != expected).then_some((actual, expected))
}
