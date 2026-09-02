//! Style/CaseLikeIf — replace case-like `if-elsif` with `case-when`.

mod common;
mod condition;
mod target;

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CaseLikeIf;

fn if_condition(node: Node<'_>) -> Option<Node<'_>> {
    if let Some(c) = node.child_by_field_name("condition") {
        return Some(c);
    }
    let mut cur = node.walk();
    for child in node.named_children(&mut cur) {
        match child.kind() {
            "then" | "else" | "elsif" => break,
            "if" | "unless" | "comment" => continue,
            _ => return Some(child),
        }
    }
    None
}

fn if_alternative(node: Node<'_>) -> Option<Node<'_>> {
    if let Some(alt) = node.child_by_field_name("alternative") {
        return Some(alt);
    }
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .find(|c| matches!(c.kind(), "elsif" | "else"))
}

fn branch_conditions(mut node: Node<'_>) -> Vec<Node<'_>> {
    let mut out = Vec::new();
    loop {
        if let Some(cond) = if_condition(node) {
            out.push(cond);
        }
        let Some(alt) = if_alternative(node) else {
            break;
        };
        if alt.kind() != "elsif" {
            break;
        }
        node = alt;
    }
    out
}

fn should_check(node: Node<'_>) -> bool {
    node.kind() == "if"
        && node.parent().is_none_or(|p| p.kind() != "elsif")
        && if_alternative(node).is_some_and(|a| a.kind() == "elsif")
}

impl Cop for CaseLikeIf {
    fn name(&self) -> &'static str {
        "Style/CaseLikeIf"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !should_check(node) {
            return;
        }
        let conditions = branch_conditions(node);
        if conditions.len() < 2 {
            return;
        }
        let Some(target) = target::find_target(source, conditions[0]) else {
            return;
        };
        if !conditions
            .iter()
            .all(|c| condition::collect_condition(source, *c, target))
        {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Convert `if-elsif` to `case-when`.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(CaseLikeIf, "cops/style/case_like_if");
}
