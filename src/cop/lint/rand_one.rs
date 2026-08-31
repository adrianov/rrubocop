use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/RandOne — `rand(1)` / `rand(-1)` always 0.
pub struct RandOne;

fn is_one(source: &SourceFile, node: Node<'_>) -> bool {
    let t = node_text(source, node).replace('_', "");
    matches!(t.as_str(), "1" | "-1" | "1.0" | "-1.0")
}

impl Cop for RandOne {
    fn name(&self) -> &'static str {
        "Lint/RandOne"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(meth) = call_method_name(source, node) else {
            return;
        };
        if meth != b"rand" && meth != b"Random.rand" {
            // also Kernel.rand via receiver
            if meth != b"rand" {
                return;
            }
        }
        if meth != b"rand" {
            return;
        }
        let args = argument_nodes(node);
        if args.len() != 1 || !is_one(source, args[0]) {
            return;
        }
        let text = node_text(source, node);
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("`{text}` always returns `0`. Perhaps you meant `rand(2)` or `rand`?"),
        ));
    }
}
