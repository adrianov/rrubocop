use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/SuppressedException — empty rescue body.
pub struct SuppressedException;

fn only_nil(stmts: &[Node<'_>]) -> bool {
    stmts.is_empty() || (stmts.len() == 1 && stmts[0].kind() == "nil")
}

fn body_empty_or_nil(node: Node<'_>) -> bool {
    if let Some(body) = node.child_by_field_name("body") {
        let mut cur = body.walk();
        let stmts: Vec<_> = body.named_children(&mut cur).collect();
        return only_nil(&stmts);
    }
    let mut cur = node.walk();
    let named: Vec<_> = node
        .named_children(&mut cur)
        .filter(|n| !matches!(n.kind(), "exceptions" | "exception_variable"))
        .collect();
    only_nil(&named)
}

impl Cop for SuppressedException {
    fn name(&self) -> &'static str {
        "Lint/SuppressedException"
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
        if !body_empty_or_nil(node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not suppress exceptions.".to_string(),
        ));
    }
}
