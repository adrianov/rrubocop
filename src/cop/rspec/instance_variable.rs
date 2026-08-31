//! RSpec/InstanceVariable — prefer `let` over instance variables in specs.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct InstanceVariable;

const MSG: &str = "Avoid instance variables — use let, a method call, or a local variable (if possible).";

impl Cop for InstanceVariable {
    fn name(&self) -> &'static str {
        "RSpec/InstanceVariable"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["instance_variable"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.kind() != "instance_variable" {
            return;
        }
        // Skip inside `def` (custom helpers / classes inside specs).
        let mut p = node.parent();
        while let Some(cur) = p {
            if matches!(cur.kind(), "method" | "singleton_method") {
                return;
            }
            p = cur.parent();
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
