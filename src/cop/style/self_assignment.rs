//! Style/SelfAssignment — prefer ||= etc.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SelfAssignment;

impl Cop for SelfAssignment {
    fn name(&self) -> &'static str {
        "Style/SelfAssignment"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["assignment"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(op) = self_assign_op(source, node) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Use self-assignment shorthand `{}=`.", String::from_utf8_lossy(op)),
        ));
    }
}

fn self_assign_op<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    let left = node.child_by_field_name("left")?;
    // RuboCop only checks lvasgn / ivasgn / cvasgn — not `obj.x = obj.x - 1`.
    if !matches!(left.kind(), "identifier" | "instance_variable" | "class_variable") {
        return None;
    }
    binary_self_op(source, left, node.child_by_field_name("right")?)
}

fn binary_self_op<'a>(
    source: &'a SourceFile,
    left: Node<'_>,
    right: Node<'_>,
) -> Option<&'a [u8]> {
    if right.kind() != "binary" {
        return None;
    }
    let mut cur = right.walk();
    let kids: Vec<_> = right.children(&mut cur).collect();
    (kids.len() >= 3 && node_bytes(source, left) == node_bytes(source, kids[0]))
        .then(|| node_bytes(source, kids[1]))
        .filter(|op| {
            matches!(
                *op,
                b"+" | b"-" | b"*" | b"/" | b"|" | b"&" | b"^" | b"<<" | b">>" | b"||" | b"&&"
            )
        })
}
