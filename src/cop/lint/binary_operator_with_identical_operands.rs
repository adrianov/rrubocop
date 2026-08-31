use tree_sitter::Node;

use crate::cop::shared::{node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/BinaryOperatorWithIdenticalOperands — `x == x`, `a || a`, etc.
pub struct BinaryOperatorWithIdenticalOperands;

const OPS: &[&[u8]] = &[
    b"==", b"===", b"!=", b"=~", b"!~", b">", b">=", b"<", b"<=", b"<=>", b"||", b"&&", b"or", b"and",
];

impl Cop for BinaryOperatorWithIdenticalOperands {
    fn name(&self) -> &'static str {
        "Lint/BinaryOperatorWithIdenticalOperands"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["binary"]
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
        let op_bytes = node_bytes(source, op);
        if !OPS.contains(&op_bytes) {
            return;
        }
        let Some(left) = node.child_by_field_name("left") else {
            return;
        };
        let Some(right) = node.child_by_field_name("right") else {
            return;
        };
        if node_bytes(source, left) != node_bytes(source, right) {
            return;
        }
        let op_s = node_text(source, op);
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Binary operator `{op_s}` has identical operands."),
        ));
    }
}
