//! Style/ParenthesesAroundCondition.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ParenthesesAroundCondition;

impl Cop for ParenthesesAroundCondition {
    fn name(&self) -> &'static str {
        "Style/ParenthesesAroundCondition"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &[
            "if", "unless", "while", "until",
            "if_modifier", "unless_modifier", "while_modifier", "until_modifier",
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(cond) = node.child_by_field_name("condition") else {
            return;
        };
        if cond.kind() != "parenthesized_statements" {
            return;
        }
        if matches!(
            node.kind(),
            "if_modifier" | "unless_modifier" | "while_modifier" | "until_modifier"
        ) {
            return;
        }
        // RuboCop AllowSafeAssignment (default true): `if (x = y)` is allowed.
        if config.get_bool("AllowSafeAssignment", true) && condition_has_assignment(cond) {
            return;
        }
        report(self, source, node, cond, diagnostics, &mut corrections);
    }
}

fn condition_has_assignment(cond: Node<'_>) -> bool {
    let mut cur = cond.walk();
    cond.named_children(&mut cur).any(|n| {
        matches!(n.kind(), "assignment" | "operator_assignment")
            || (n.kind() == "parenthesized_statements" && condition_has_assignment(n))
    })
}

fn keyword_article(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "while" | "while_modifier" => ("a", "while"),
        "until" | "until_modifier" => ("an", "until"),
        "unless" | "unless_modifier" => ("an", "unless"),
        _ => ("an", "if"),
    }
}

fn report(
    cop: &ParenthesesAroundCondition,
    source: &SourceFile,
    node: Node<'_>,
    cond: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (article, kw) = keyword_article(node.kind());
    let (line, col) = source.offset_to_line_col(cond.start_byte());
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        format!("Don't use parentheses around the condition of {article} `{kw}`."),
    );
    if let Some(corr) = corrections.as_mut() {
        strip_open(cop, source, cond, corr);
        strip_close(cop, source, cond, corr);
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn strip_open(
    cop: &ParenthesesAroundCondition,
    source: &SourceFile,
    cond: Node<'_>,
    corr: &mut Vec<Correction>,
) {
    if source.as_bytes().get(cond.start_byte()) != Some(&b'(') {
        return;
    }
    corr.push(Correction {
        start: cond.start_byte(),
        end: cond.start_byte() + 1,
        replacement: String::new(),
        cop_name: cop.name(),
        cop_index: 0,
    });
}

fn strip_close(
    cop: &ParenthesesAroundCondition,
    source: &SourceFile,
    cond: Node<'_>,
    corr: &mut Vec<Correction>,
) {
    if cond.end_byte() <= cond.start_byte() {
        return;
    }
    if source.as_bytes().get(cond.end_byte() - 1) != Some(&b')') {
        return;
    }
    corr.push(Correction {
        start: cond.end_byte() - 1,
        end: cond.end_byte(),
        replacement: String::new(),
        cop_name: cop.name(),
        cop_index: 0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ParenthesesAroundCondition, "cops/style/parentheses_around_condition");
}
