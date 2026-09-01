//! Style/RedundantBegin — begin without rescue/ensure/else.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RedundantBegin;

impl Cop for RedundantBegin {
    fn name(&self) -> &'static str {
        "Style/RedundantBegin"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["begin"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_offensive_begin(node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Redundant `begin` block detected.".to_string(),
        ));
    }
}

fn is_offensive_begin(node: Node<'_>) -> bool {
    // RuboCop on_def / on_block: body is entirely a kwbegin.
    if is_sole_callable_body(node) {
        return true;
    }
    // RuboCop on_kwbegin: flag when not allowable.
    !allowable_kwbegin(node)
}

fn is_sole_callable_body(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if !matches!(parent.kind(), "body_statement" | "block_body") {
        return false;
    }
    let mut cur = parent.walk();
    let named: Vec<_> = parent.named_children(&mut cur).collect();
    if named.len() != 1 || named[0].id() != node.id() {
        return false;
    }
    let Some(callable) = parent.parent() else {
        return false;
    };
    match callable.kind() {
        "method" | "singleton_method" | "do_block" | "lambda" => true,
        // RuboCop `on_block`: skip brace blocks.
        "block" => false,
        _ => false,
    }
}

fn allowable_kwbegin(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    let kids: Vec<_> = node.named_children(&mut cur).collect();
    if kids.is_empty() {
        return true;
    }
    if kids.iter().any(|k| matches!(k.kind(), "rescue" | "ensure" | "else")) {
        return true;
    }
    // RuboCop: two or more body statements → keep begin.
    if kids.len() >= 2 {
        return true;
    }
    valid_context_using_only_begin(node)
}

fn valid_context_using_only_begin(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    matches!(
        parent.kind(),
        "assignment"
            | "operator_assignment"
            | "binary" // and/or
            | "call"
            | "while"
            | "until"
            | "while_modifier"
            | "until_modifier"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RedundantBegin, "cops/style/redundant_begin");
}
