//! Style/CaseEquality — avoid ===.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CaseEquality;

impl Cop for CaseEquality {
    fn name(&self) -> &'static str {
        "Style/CaseEquality"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["binary", "call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let hit = if node.kind() == "call" {
            call_method_name(source, node) == Some(b"===")
        } else {
            let mut cur = node.walk();
            node.children(&mut cur).any(|c| node_bytes(source, c) == b"===")
        };
        if !hit {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Avoid the use of the case equality operator `===`.".to_string(),
        ));
    }
}
