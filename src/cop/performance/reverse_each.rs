//! Performance/ReverseEach — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, argument_nodes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct ReverseEach;


impl Cop for ReverseEach {
    fn name(&self) -> &'static str {
        "Performance/ReverseEach"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if call_method_name(source, node) != Some(b"each") { return; }
        let Some(recv) = call_receiver(node) else { return; };
        if !matches!(recv.kind(), "call" | "command") { return; }
        if call_method_name(source, recv) != Some(b"reverse") { return; }
        if !argument_nodes(recv).is_empty() { return; }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source, line, col,
            "Use `reverse_each` instead of `reverse.each`.".to_string(),
        ));
    }
}
