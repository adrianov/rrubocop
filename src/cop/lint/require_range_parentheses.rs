//! Lint/RequireRangeParentheses — range end on a following line without parens.

use tree_sitter::Node;

use crate::cop::shared::{node_line, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct RequireRangeParentheses;

impl Cop for RequireRangeParentheses {
    fn name(&self) -> &'static str {
        "Lint/RequireRangeParentheses"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["range"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !needs_parens(source, node) {
            return;
        }
        let op = range_operator(source, node);
        let begin = node
            .child_by_field_name("begin")
            .map(|n| node_text(source, n))
            .unwrap_or_default();
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!(
                "Wrap the range literal `{begin}{op}` in parentheses to avoid confusion with an endless range."
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RequireRangeParentheses, "cops/lint/require_range_parentheses");
}

fn needs_parens(source: &SourceFile, node: Node<'_>) -> bool {
    if node
        .parent()
        .is_some_and(|p| p.kind() == "parenthesized_statements")
    {
        return false;
    }
    let Some(begin) = node.child_by_field_name("begin") else {
        return false;
    };
    let Some(end) = node.child_by_field_name("end") else {
        return false;
    };
    let Some(op_line) = operator_line(source, node, begin) else {
        return false;
    };
    op_line != node_line(source, end)
}

fn operator_line(source: &SourceFile, range: Node<'_>, begin: Node<'_>) -> Option<usize> {
    let mut cur = range.walk();
    for ch in range.children(&mut cur) {
        if !ch.is_named() && matches!(ch.kind(), ".." | "...") {
            return Some(node_line(source, ch));
        }
    }
    // Fallback: operator sits between begin and end.
    Some(node_line(source, begin))
}

fn range_operator(source: &SourceFile, range: Node<'_>) -> &'static str {
    let mut cur = range.walk();
    for ch in range.children(&mut cur) {
        if !ch.is_named() {
            match ch.kind() {
                "..." => return "...",
                ".." => return "..",
                _ => {}
            }
        }
    }
    let t = node_text(source, range);
    if t.contains("...") {
        "..."
    } else {
        ".."
    }
}
