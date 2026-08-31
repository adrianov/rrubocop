use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/NestedMethodDefinition — method defined inside method.
pub struct NestedMethodDefinition;

fn inside_method(node: Node<'_>) -> bool {
    let mut cur = node.parent();
    while let Some(p) = cur {
        match p.kind() {
            "method" | "singleton_method" => return true,
            "class" | "module" | "singleton_class" => return false,
            _ => cur = p.parent(),
        }
    }
    false
}

impl Cop for NestedMethodDefinition {
    fn name(&self) -> &'static str {
        "Lint/NestedMethodDefinition"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !inside_method(node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Method definitions must not be nested. Use `lambda` instead.".to_string(),
        ));
    }
}
