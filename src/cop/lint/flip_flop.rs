//! Lint/FlipFlop — avoid flip-flop operators (`..` / `...` as conditions).

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct FlipFlop;

impl Cop for FlipFlop {
    fn name(&self) -> &'static str {
        "Lint/FlipFlop"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if", "unless", "while", "until"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(cond) = node.child_by_field_name("condition") else {
            return;
        };
        if cond.kind() != "range" {
            return;
        }
        let (line, col) = source.offset_to_line_col(cond.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Avoid the use of flip-flop operators.".to_string(),
        ));
    }
}
