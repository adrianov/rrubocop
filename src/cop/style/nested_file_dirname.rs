//! Style/NestedFileDirname — avoid nested File.dirname.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, is_const_named};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct NestedFileDirname;

impl Cop for NestedFileDirname {
    fn name(&self) -> &'static str {
        "Style/NestedFileDirname"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"dirname"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_nested_dirname(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Use `File.dirname(path, n)` instead of nested `File.dirname`.".to_string(),
        ));
    }
}

fn is_nested_dirname(source: &SourceFile, node: Node<'_>) -> bool {
    if call_method_name(source, node) != Some(b"dirname") {
        return false;
    }
    if !call_receiver(node).is_some_and(|r| is_const_named(source, r, b"File")) {
        return false;
    }
    let args = argument_nodes(node);
    let Some(arg) = args.first() else {
        return false;
    };
    arg.kind() == "call" && call_method_name(source, *arg) == Some(b"dirname")
}
