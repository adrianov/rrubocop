use tree_sitter::Node;

use crate::cop::shared::{call_method_name, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/UselessRescue — rescue that only re-raises.
pub struct UselessRescue;

fn is_raise(source: &SourceFile, node: Node<'_>) -> bool {
    match node.kind() {
        "identifier" => node_bytes(source, node) == b"raise",
        "call" => call_method_name(source, node) == Some(b"raise"),
        _ => false,
    }
}

fn rescue_body(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("body").or_else(|| {
        let mut cur = node.walk();
        node.named_children(&mut cur)
            .find(|n| matches!(n.kind(), "then" | "body_statement"))
    })
}

impl Cop for UselessRescue {
    fn name(&self) -> &'static str {
        "Lint/UselessRescue"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["rescue"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(body) = rescue_body(node) else {
            return;
        };
        let mut cur = body.walk();
        let stmts: Vec<_> = body.named_children(&mut cur).collect();
        if stmts.len() != 1 || !is_raise(source, stmts[0]) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Useless `rescue` detected.".to_string(),
        ));
    }
}
