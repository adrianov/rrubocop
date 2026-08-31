use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/RescueException — rescue Exception / ::Exception.
pub struct RescueException;

fn is_exception_const(source: &SourceFile, node: Node<'_>) -> bool {
    match node.kind() {
        "constant" => node_bytes(source, node) == b"Exception",
        "scope_resolution" => {
            let name = node.child_by_field_name("name");
            let scope = node.child_by_field_name("scope");
            name.map(|n| node_bytes(source, n) == b"Exception").unwrap_or(false)
                && scope.is_none()
        }
        _ => false,
    }
}

impl Cop for RescueException {
    fn name(&self) -> &'static str {
        "Lint/RescueException"
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
        let Some(exceptions) = node.child_by_field_name("exceptions") else {
            return;
        };
        let mut cur = exceptions.walk();
        let hit = exceptions
            .named_children(&mut cur)
            .any(|e| is_exception_const(source, e));
        if !hit {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Avoid rescuing the `Exception` class. Perhaps you meant to rescue `StandardError`?"
                .to_string(),
        ));
    }
}
