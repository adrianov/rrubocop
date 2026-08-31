//! Rails/RedundantReceiverInWithOptions — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct RedundantReceiverInWithOptions;

const MSG: &str = "Redundant receiver in `with_options`.";

impl Cop for RedundantReceiverInWithOptions {
    fn name(&self) -> &'static str {
        "Rails/RedundantReceiverInWithOptions"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["block", "call", "identifier", "body_statement", "command"]
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
        const METHODS: &[&[u8]] = &[b"with_options", b"_1", b"it"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
