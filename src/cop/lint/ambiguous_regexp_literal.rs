use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/AmbiguousRegexpLiteral — regexp as first arg without parentheses.
pub struct AmbiguousRegexpLiteral;

impl Cop for AmbiguousRegexpLiteral {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousRegexpLiteral"
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
        let Some(args_node) = node.child_by_field_name("arguments") else {
            return;
        };
        if node_bytes(source, args_node).starts_with(b"(") {
            return;
        }
        let args = argument_nodes(node);
        let Some(first) = args.first() else {
            return;
        };
        if first.kind() != "regex" {
            return;
        }
        // RuboCop only flags slash regexps (`/…/`), not `%r{…}`.
        if node_bytes(source, *first).starts_with(b"%") {
            return;
        }
        let (line, col) = source.offset_to_line_col(first.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Ambiguous regexp literal. Parenthesize the method arguments if it's surely a regexp literal, or add a whitespace to the right of the `/` if it should be a division.".to_string(),
        ));
    }
}
