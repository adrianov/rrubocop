use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/ElseLayout — odd else layout (statement on else line).
pub struct ElseLayout;

fn body_stmts<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .filter(|n| n.kind() != "comment")
        .collect()
}

fn same_line(source: &SourceFile, a: Node<'_>, b: Node<'_>) -> bool {
    source.offset_to_line_col(a.start_byte()).0 == source.offset_to_line_col(b.start_byte()).0
}

impl Cop for ElseLayout {
    fn name(&self) -> &'static str {
        "Lint/ElseLayout"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["else"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Odd layout: first body stmt shares line with `else` keyword.
        // Trailing comments (`else # note`) are not body statements.
        let named = body_stmts(node);
        let Some(&first) = named.first() else {
            return;
        };
        if !same_line(source, node, first) || named.len() < 2 {
            return;
        }
        let (line, col) = source.offset_to_line_col(first.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Odd `else` layout detected. Did you mean to use `elsif`?".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ElseLayout, "cops/lint/else_layout");
}
