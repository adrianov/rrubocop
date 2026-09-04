//! Layout/CaseIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CaseIndentation;

fn case_keyword_col(source: &SourceFile, case_node: Node<'_>) -> usize {
    let mut cur = case_node.walk();
    case_node
        .children(&mut cur)
        .find(|c| !c.is_named() && c.kind() == "case")
        .map(|kw| shared::node_col(source, kw))
        .unwrap_or_else(|| shared::node_col(source, case_node))
}

fn expected_col(source: &SourceFile, case_node: Node<'_>, style: &str, width: usize) -> usize {
    let case_col = case_keyword_col(source, case_node);
    match style {
        "end" => shared::end_keyword(case_node)
            .map(|e| shared::node_col(source, e))
            .unwrap_or(case_col),
        "case" => case_col,
        _ => case_col + width,
    }
}

fn branch_msg(style: &str, branch: &str) -> String {
    match style {
        "end" => format!("Indent `{branch}` as deep as `end`."),
        "case" => format!("Indent `{branch}` as deep as `case`."),
        _ => format!("Indent `{branch}` one step more than `case`."),
    }
}

fn is_case_branch(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    matches!(parent.kind(), "case" | "case_match")
        && !(node.kind() == "else" && parent.kind() == "when")
}

/// RuboCop `end_and_last_conditional_same_line?` — compact trailing branch/`end`.
fn end_and_last_conditional_same_line(source: &SourceFile, case_node: Node<'_>) -> bool {
    let Some(end_kw) = shared::end_keyword(case_node) else {
        return false;
    };
    let end_line = shared::node_line(source, end_kw);
    let mut cur = case_node.walk();
    case_node
        .named_children(&mut cur)
        .filter(|c| matches!(c.kind(), "when" | "else" | "in"))
        .last()
        .is_some_and(|b| shared::node_line(source, b) == end_line)
}

impl Cop for CaseIndentation {
    fn name(&self) -> &'static str {
        "Layout/CaseIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["when", "else"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<Correction>>,
    ) {
        if !should_check_branch(source, node, config) {
            return;
        }
        let parent = node.parent().unwrap();
        let style = config.get_str("EnforcedStyle", "case");
        let expected = expected_col(
            source,
            parent,
            style,
            config.get_usize("IndentationWidth", 2),
        );
        if shared::node_col(source, node) == expected {
            return;
        }
        report_branch(self, source, node, parent, style, expected, diagnostics, corrections);
    }
}

fn report_branch(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    parent: Node<'_>,
    style: &str,
    expected: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: Option<&mut Vec<Correction>>,
) {
    // Same-line `when` — mid-line indent autocorrect is not meaningful.
    let same_line = shared::node_line(source, node) == shared::node_line(source, parent);
    let mut corr = if same_line { None } else { corrections };
    report::fix_indent(
        cop,
        source,
        node.start_byte(),
        branch_msg(style, node.kind()),
        diagnostics,
        &mut corr,
        shared::line_indent(source, node.start_byte()),
        expected,
    );
}

fn should_check_branch(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if !is_case_branch(node) {
        return false;
    }
    let parent = node.parent().unwrap();
    if skip_case_branch(source, parent, config) {
        return false;
    }
    // Compact `case n when …` — RuboCop only flags the same-line `when`,
    // not the following `else` (ElseAlignment owns that).
    !(node.kind() == "else" && case_has_same_line_when(source, parent))
}

fn case_has_same_line_when(source: &SourceFile, case_node: Node<'_>) -> bool {
    let case_line = shared::node_line(source, case_node);
    let mut cur = case_node.walk();
    case_node
        .named_children(&mut cur)
        .filter(|c| c.kind() == "when")
        .any(|w| shared::node_line(source, w) == case_line)
}

/// RuboCop skips single-line `case … when … end` and same-line end/last when.
fn skip_case_branch(source: &SourceFile, parent: Node<'_>, config: &CopConfig) -> bool {
    let case_line = shared::node_line(source, parent);
    if case_line
        == shared::end_keyword(parent)
            .map(|e| shared::node_line(source, e))
            .unwrap_or(case_line)
    {
        return true;
    }
    config.get_str("EnforcedStyle", "case") == "end"
        && end_and_last_conditional_same_line(source, parent)
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(CaseIndentation, "cops/layout/case_indentation");
}
