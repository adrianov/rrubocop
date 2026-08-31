//! Rails/IndexBy — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct IndexBy;

const MSG: &str = "Use `index_by` instead of `map ....to_h`.";

impl Cop for IndexBy {
    fn name(&self) -> &'static str {
        "Rails/IndexBy"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["array", "block", "call", "hash", "identifier", "body_statement", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        const METHODS: &[&[u8]] = &[b"_1", b"it", b"each_with_object", b"to_h", b"map", b"Hash"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
