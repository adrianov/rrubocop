//! Style/ParallelAssignment — avoid parallel assignment (breadth).

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ParallelAssignment;

impl Cop for ParallelAssignment {
    fn name(&self) -> &'static str {
        "Style/ParallelAssignment"
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
        let Some(left) = node.child_by_field_name("left") else {
            return;
        };
        if left.kind() != "left_assignment_list" && left.kind() != "destructured_left_assignment" {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not use parallel assignment.".to_string(),
        ));
    }
}
