use tree_sitter::Node;

use crate::cop::shared::{call_method_name, for_each_descendant};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/NextWithoutAccumulator — bare `next` in reduce/inject.
pub struct NextWithoutAccumulator;

impl Cop for NextWithoutAccumulator {
    fn name(&self) -> &'static str {
        "Lint/NextWithoutAccumulator"
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
        if meth != b"reduce" && meth != b"inject" {
            return;
        }
        let Some(block) = node.child_by_field_name("block") else {
            return;
        };
        for_each_descendant(block, |n| {
            if n.kind() != "next" {
                return;
            }
            // next without arguments
            if n.named_child_count() > 0 {
                return;
            }
            // skip nested blocks' reduce — breadth-first: flag all bare next in this block
            let (line, col) = source.offset_to_line_col(n.start_byte());
            diagnostics.push(self.diagnostic(
                source,
                line,
                col,
                "Use `next` with an accumulator argument in a `reduce`.".to_string(),
            ));
        });
    }
}
