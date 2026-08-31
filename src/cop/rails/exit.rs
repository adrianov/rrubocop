//! Rails/Exit — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, is_const_named};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct Exit;

fn is_exit_name(method: &[u8]) -> bool {
    // RuboCop Rails/Exit: only `exit` / `exit!` (not `abort`).
    matches!(method, b"exit" | b"exit!")
}

fn allowed_receiver(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(recv) = call_receiver(node) else {
        return true;
    };
    is_const_named(source, recv, b"Kernel") || is_const_named(source, recv, b"Process")
}

fn exit_method<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    let method = call_method_name(source, node)?;
    if !is_exit_name(method) || !allowed_receiver(source, node) {
        return None;
    }
    if argument_nodes(node).len() > 1 {
        return None;
    }
    Some(method)
}

impl Cop for Exit {
    fn name(&self) -> &'static str {
        "Rails/Exit"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/app/**/*.rb", "**/config/**/*.rb", "**/lib/**/*.rb"]
    }

    fn default_exclude(&self) -> &'static [&'static str] {
        &["**/lib/**/*.rake"]
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
        let Some(method) = exit_method(source, node) else {
            return;
        };
        let name = std::str::from_utf8(method).unwrap_or("exit");
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Do not use `{name}` in Rails applications."),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(Exit, "cops/rails/exit");
}
