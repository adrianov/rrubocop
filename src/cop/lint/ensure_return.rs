//! Lint/EnsureReturn — `return` inside `ensure`.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct EnsureReturn;

fn find_return_offsets(node: Node<'_>, out: &mut Vec<usize>) {
    if node.kind() == "return" {
        out.push(node.start_byte());
    }
    if matches!(node.kind(), "method" | "singleton_method" | "class" | "module") {
        return;
    }
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        find_return_offsets(child, out);
    }
}

impl Cop for EnsureReturn {
    fn name(&self) -> &'static str {
        "Lint/EnsureReturn"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["ensure"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut offsets = Vec::new();
        find_return_offsets(node, &mut offsets);
        for off in offsets {
            let (line, col) = source.offset_to_line_col(off);
            diagnostics.push(self.diagnostic(
                source,
                line,
                col,
                "Do not return from an `ensure` block.".to_string(),
            ));
        }
    }
}
