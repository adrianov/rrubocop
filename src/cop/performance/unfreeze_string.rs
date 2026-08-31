//! Performance/UnfreezeString — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, is_const_named, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct UnfreezeString;

fn empty_string_literal(source: &SourceFile, recv: Node<'_>) -> bool {
    matches!(node_text(source, recv).as_str(), "''" | "\"\"")
}

fn is_unfreeze(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(method) = call_method_name(source, node) else {
        return false;
    };
    let Some(recv) = call_receiver(node) else {
        return false;
    };
    match method {
        b"new" | b"to_s" | b"to_str" => is_const_named(source, recv, b"String"),
        b"dup" | b"clone" => empty_string_literal(source, recv),
        _ => false,
    }
}

impl Cop for UnfreezeString {
    fn name(&self) -> &'static str {
        "Performance/UnfreezeString"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "scope_resolution", "constant", "string", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_unfreeze(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Prefer unary plus to get an unfrozen string literal.".to_string(),
        ));
    }
}
