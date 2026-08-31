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
    let right = node.child_by_field_name("right")?;
    if right.kind() != "binary" {
        return None;
    }
    let mut cur = right.walk();
    let kids: Vec<_> = right.children(&mut cur).collect();
    if kids.len() < 3 {
        return None;
    }
    if node_bytes(source, left) != node_bytes(source, kids[0]) {
        return None;
    }
    let op = node_bytes(source, kids[1]);
    matches!(
        op,
        b"+" | b"-" | b"*" | b"/" | b"|" | b"&" | b"^" | b"<<" | b">>" | b"||" | b"&&"
    )
    .then_some(op)
}
