use tree_sitter::Node;

use crate::cop::shared::{call_method_name, for_each_descendant};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/NonLocalExitFromIterator — bare return in iterator block.
pub struct NonLocalExitFromIterator;

const ITERATORS: &[&[u8]] = &[
    b"each", b"map", b"collect", b"select", b"find_all", b"reject", b"detect", b"find",
    b"any?", b"all?", b"none?", b"one?", b"times", b"loop", b"upto", b"downto", b"step",
];

impl Cop for NonLocalExitFromIterator {
    fn name(&self) -> &'static str {
        "Lint/NonLocalExitFromIterator"
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
        if !ITERATORS.contains(&meth) {
            return;
        }
        let Some(block) = node.child_by_field_name("block") else {
            return;
        };
        for_each_descendant(block, |n| {
            if n.kind() != "return" {
                return;
            }
            if n.named_child_count() > 0 {
                return; // return with value is ok-ish / different
            }
            let (line, col) = source.offset_to_line_col(n.start_byte());
            diagnostics.push(self.diagnostic(
                source,
                line,
                col,
                "Non-local exit from iterator, without return value. `next`, `break`, `Array#find`, etc. are preferred.".to_string(),
            ));
        });
    }
}
