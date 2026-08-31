//! Style/DirEmpty — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, is_const_named, node_text};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct DirEmpty;

impl Cop for DirEmpty {
    fn name(&self) -> &'static str {
        "Style/DirEmpty"
    }

    fn supports_autocorrect(&self) -> bool {
        true
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
        corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        const METHODS: &[&[u8]] = &[b"empty?"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            "Use `Dir.empty?` for directory emptiness checks.".to_string(),
        );
        if let Some(corr) = corrections {
            if let Some(replacement) = dir_empty_replacement(source, node) {
                corr.push(Correction {
                    start: node.start_byte(),
                    end: node.end_byte(),
                    replacement,
                    cop_name: self.name(),
                    cop_index: 0,
                });
                diag.corrected = true;
            }
        }
        diagnostics.push(diag);
    }
}

/// `Dir.children(path).empty?` / `Dir.each_child(path).empty?` → `Dir.empty?(path)`.
fn dir_empty_replacement(source: &SourceFile, node: Node<'_>) -> Option<String> {
    let recv = call_receiver(node)?;
    let recv_method = call_method_name(source, recv)?;
    if recv_method != b"children" && recv_method != b"each_child" {
        return None;
    }
    let dir_recv = call_receiver(recv)?;
    if !is_const_named(source, dir_recv, b"Dir") {
        return None;
    }
    let dir_src = node_text(source, dir_recv);
    let args_part = match recv.child_by_field_name("arguments") {
        Some(a) => node_text(source, a),
        None => "()".to_string(),
    };
    Some(format!("{dir_src}.empty?{args_part}"))
}
