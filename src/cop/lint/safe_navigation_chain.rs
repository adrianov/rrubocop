use tree_sitter::Node;

use crate::cop::shared::{call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/SafeNavigationChain — `x&.y.z` ordinary call after &.
pub struct SafeNavigationChain;

fn has_safe_nav(source: &SourceFile, mut node: Node<'_>) -> bool {
    loop {
        if node.kind() != "call" {
            return false;
        }
        if let Some(op) = node.child_by_field_name("operator") {
            if node_bytes(source, op) == b"&." {
                return true;
            }
        }
        match call_receiver(node) {
            Some(r) => node = r,
            None => return false,
        }
    }
}

impl Cop for SafeNavigationChain {
    fn name(&self) -> &'static str {
        "Lint/SafeNavigationChain"
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
        let Some(op) = node.child_by_field_name("operator") else {
            return;
        };
        // Flag `.` call whose receiver chain contains `&.`
        if node_bytes(source, op) != b"." {
            return;
        }
        let Some(recv) = call_receiver(node) else {
            return;
        };
        if !has_safe_nav(source, recv) {
            return;
        }
        let (line, col) = source.offset_to_line_col(op.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not chain ordinary method call after safe navigation operator.".to_string(),
        ));
    }
}
