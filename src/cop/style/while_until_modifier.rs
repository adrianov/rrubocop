//! Style/WhileUntilModifier — prefer modifier form for single-line bodies.

use tree_sitter::Node;

use crate::cop::shared::{node_line, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct WhileUntilModifier;

impl Cop for WhileUntilModifier {
    fn name(&self) -> &'static str {
        "Style/WhileUntilModifier"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["while", "until"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !single_line_as_modifier(source, node, config) {
            return;
        }
        let kw = node.kind();
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Favor modifier `{kw}` usage when having a single-line body."),
        ));
    }
}

fn single_line_as_modifier(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    let Some(body) = effective_body(node) else {
        return false;
    };
    if !body_single_line(source, body) {
        return false;
    }
    if condition_has_assignment(node) {
        return false;
    }
    if config.get_bool("Layout/LineLength/Enabled", false) {
        modifier_fits(source, node, body, config)
    } else {
        true
    }
}

fn effective_body(node: Node<'_>) -> Option<Node<'_>> {
    let body = node.child_by_field_name("body")?;
    if body.kind() == "do" || body.kind() == "begin" {
        let mut cur = body.walk();
        let stmts: Vec<_> = body
            .named_children(&mut cur)
            .filter(|c| !matches!(c.kind(), "comment" | "rescue" | "ensure"))
            .collect();
        if stmts.len() == 1 {
            return Some(stmts[0]);
        }
        return None;
    }
    Some(body)
}

fn body_single_line(source: &SourceFile, body: Node<'_>) -> bool {
    node_line(source, body)
        == source
            .offset_to_line_col(body.end_byte().saturating_sub(1))
            .0
}

fn condition_has_assignment(node: Node<'_>) -> bool {
    let Some(cond) = node.child_by_field_name("condition") else {
        return false;
    };
    let mut stack = vec![cond];
    while let Some(n) = stack.pop() {
        if matches!(n.kind(), "assignment" | "operator_assignment") {
            return true;
        }
        let mut cur = n.walk();
        for child in n.named_children(&mut cur) {
            stack.push(child);
        }
    }
    false
}

fn modifier_fits(source: &SourceFile, node: Node<'_>, body: Node<'_>, config: &CopConfig) -> bool {
    let max = config.get_usize("Layout/LineLength/Max", 120);
    let kw = node.kind();
    let cond = node_text(source, node.child_by_field_name("condition").unwrap());
    let body_s = node_text(source, body);
    let line = node_line(source, node);
    let prefix = source.line_text(line).map(str::len).unwrap_or(0);
    prefix + body_s.len() + 1 + kw.len() + 1 + cond.len() <= max
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(WhileUntilModifier, "cops/style/while_until_modifier");
}
