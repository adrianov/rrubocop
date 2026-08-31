use tree_sitter::Node;

use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/RedundantSplatExpansion — `*[1,2]` / `*%w[...]`.
pub struct RedundantSplatExpansion;

const LIT_KINDS: &[&str] = &[
    "integer",
    "float",
    "string",
    "simple_symbol",
    "true",
    "false",
    "nil",
    "bare_string",
    "bare_symbol",
    "hash",
    "array",
];

fn splat_inner(node: Node<'_>) -> Option<Node<'_>> {
    let mut cur = node.walk();
    let inner = node.named_children(&mut cur).next()?;
    matches!(inner.kind(), "array" | "string_array" | "symbol_array").then_some(inner)
}

fn all_literals(inner: Node<'_>) -> bool {
    let mut cur = inner.walk();
    inner
        .named_children(&mut cur)
        .all(|e| LIT_KINDS.contains(&e.kind()))
}

fn in_brackets(source: &SourceFile, node: Node<'_>) -> bool {
    node.parent()
        .and_then(|p| p.parent())
        .is_some_and(|p| {
            let t = node_text(source, p);
            t.contains('[') || t.contains('(')
        })
}

impl Cop for RedundantSplatExpansion {
    fn name(&self) -> &'static str {
        "Lint/RedundantSplatExpansion"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["splat_argument", "splat"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(inner) = splat_inner(node) else {
            return;
        };
        if !all_literals(inner) {
            return;
        }
        let msg = if in_brackets(source, node) {
            "Pass array contents as separate arguments."
        } else {
            "Replace splat expansion with comma separated values."
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, msg.to_string()));
    }
}
