use tree_sitter::Node;

use crate::cop::shared::{call_method_name, for_each_descendant};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/NextWithoutAccumulator — bare `next` in reduce/inject.
pub struct NextWithoutAccumulator;

impl Cop for NextWithoutAccumulator {
    fn name(&self) -> &'static str {
        "Lint/NextWithoutAccumulator"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(meth) = call_method_name(source, node) else {
            return;
        };
        if meth != b"reduce" && meth != b"inject" {
            return;
        }
        let Some(block) = node.child_by_field_name("block") else {
            return;
        };
        for_each_descendant(block, |n| {
            if let Some(diag) = bare_next_offense(self, source, n, block) {
                diagnostics.push(diag);
            }
        });
    }
}

fn bare_next_offense(
    cop: &NextWithoutAccumulator,
    source: &SourceFile,
    n: Node<'_>,
    block: Node<'_>,
) -> Option<Diagnostic> {
    if n.kind() != "next" {
        return None;
    }
    // tree-sitter nests `next` / `next` — only flag the outer node.
    if n.parent().is_some_and(|p| p.kind() == "next") {
        return None;
    }
    // RuboCop: only bare `next` (no value). `next acc if …` is fine.
    if next_has_value(source, n) {
        return None;
    }
    // Only the reduce's own block — not nested blocks.
    if parent_block(n).is_some_and(|b| b.id() != block.id()) {
        return None;
    }
    let (line, col) = source.offset_to_line_col(n.start_byte());
    Some(cop.diagnostic(
        source,
        line,
        col,
        "Use `next` with an accumulator argument in a `reduce`.".to_string(),
    ))
}

fn next_named_has_value(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|c| c.kind() != "next" && c.kind() != "comment")
}

fn next_line_has_value(source: &SourceFile, node: Node<'_>) -> bool {
    let (line, _) = source.offset_to_line_col(node.start_byte());
    let Some(text) = source.line_text(line) else {
        return false;
    };
    let line_start = source.line_start(line).unwrap_or(0);
    let byte_col = node.start_byte().saturating_sub(line_start);
    let rest = text.get(byte_col..).unwrap_or(text);
    let after = rest.strip_prefix("next").unwrap_or("");
    after_next_has_value(after)
}

fn after_next_has_value(after: &str) -> bool {
    let before_mod = after
        .split_once(" if ")
        .or_else(|| after.split_once("\tif "))
        .or_else(|| after.split_once(" unless "))
        .map(|(a, _)| a)
        .unwrap_or(after);
    before_mod.chars().any(|c| !c.is_whitespace())
}

fn next_has_value(source: &SourceFile, node: Node<'_>) -> bool {
    // Ignore nested token `next` child when counting value children.
    next_named_has_value(node) || next_line_has_value(source, node)
}

fn parent_block(node: Node<'_>) -> Option<Node<'_>> {
    let mut p = node.parent();
    while let Some(n) = p {
        if matches!(n.kind(), "block" | "do_block") {
            return Some(n);
        }
        p = n.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(NextWithoutAccumulator, "cops/lint/next_without_accumulator");
}
