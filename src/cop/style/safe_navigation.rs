//! Style/SafeNavigation — prefer &. over explicit nil checks (breadth).

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SafeNavigation;

impl Cop for SafeNavigation {
    fn name(&self) -> &'static str {
        "Style/SafeNavigation"
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
        if !is_and_safe_nav(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Use safe navigation (`&.`) instead of checking `nil` with `&&`.".to_string(),
        ));
    }
}

fn is_and_safe_nav(source: &SourceFile, node: Node<'_>) -> bool {
    let mut cur = node.walk();
    let kids: Vec<_> = node.children(&mut cur).collect();
    if kids.len() < 3 {
        return false;
    }
    if node_bytes(source, kids[1]) != b"&&" {
        return false;
    }
    let right = kids[2];
    if right.kind() != "call" {
        return false;
    }
    let Some(recv) = right.child_by_field_name("receiver") else {
        return false;
    };
    node_bytes(source, kids[0]) == node_bytes(source, recv)
}
