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
    let last_branch = case_node
        .named_children(&mut cur)
        .filter(|c| matches!(c.kind(), "when" | "else" | "in"))
        .last();
    last_branch.is_some_and(|b| shared::node_line(source, b) == end_line)
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
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if !is_case_branch(node) {
            return;
        }
        let parent = node.parent().unwrap();
        if skip_case_branch(source, parent, config) {
            return;
        }
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
        report::fix_indent(
            self,
            source,
            node.start_byte(),
            branch_msg(style, node.kind()),
            diagnostics,
            &mut corrections,
            shared::line_indent(source, node.start_byte()),
            expected,
        );
    }
}

/// RuboCop skips single-line `case … when … end` and same-line end/last when.
fn skip_case_branch(source: &SourceFile, parent: Node<'_>, config: &CopConfig) -> bool {
    let case_line = shared::node_line(source, parent);
    let end_line = shared::end_keyword(parent)
        .map(|e| shared::node_line(source, e))
        .unwrap_or(case_line);
    if case_line == end_line {
        return true;
    }
    let style = config.get_str("EnforcedStyle", "case");
    style == "end" && end_and_last_conditional_same_line(source, parent)
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(CaseIndentation, "cops/layout/case_indentation");
}
