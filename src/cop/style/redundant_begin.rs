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
        // tree-sitter `begin` for `begin...end`. Skip if it has rescue/ensure.
        let mut has_rescue_or_ensure = false;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "rescue" | "ensure" | "else") {
                has_rescue_or_ensure = true;
                break;
            }
        }
        if has_rescue_or_ensure {
            return;
        }
        // Only flag when parent is a block body (method/block) where begin is redundant
        let Some(parent) = node.parent() else {
            return;
        };
        if !matches!(
            parent.kind(),
            "method" | "singleton_method" | "block" | "do_block" | "lambda"
        ) {
            // Also body field of method
            if parent.kind() != "body_statement" {
                return;
            }
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
