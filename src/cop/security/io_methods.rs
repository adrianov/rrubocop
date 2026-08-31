//! Security/IoMethods — flag IO.read/write/… (not ::IO).

use tree_sitter::Node;

use crate::cop::shared::{
    argument_nodes, call_method_name, call_receiver, node_bytes, push_replace,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct IoMethods;

const DANGEROUS: &[&[u8]] = &[
    b"read",
    b"write",
    b"binread",
    b"binwrite",
    b"foreach",
    b"readlines",
];

impl Cop for IoMethods {
    fn name(&self) -> &'static str {
        "Security/IoMethods"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn safe_autocorrect(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "command_call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(method) = dangerous_io_method(source, node) else {
            return;
        };
        report(self, source, node, method, diagnostics, &mut corrections);
    }
}

fn report(
    cop: &IoMethods,
    source: &SourceFile,
    node: Node<'_>,
    method: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let method_str = std::str::from_utf8(method).unwrap_or("");
    let recv = call_receiver(node).unwrap_or(node);
    let meth_node = node.child_by_field_name("method").unwrap_or(node);
    let (line, column) = source.offset_to_line_col(meth_node.start_byte());
    let mut diag = cop.diagnostic(
        source,
        line,
        column,
        format!("`File.{method_str}` is safer than `IO.{method_str}`."),
    );
    if push_replace(
        corrections,
        recv.start_byte(),
        recv.end_byte(),
        "File",
        cop.name(),
    ) {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn dangerous_io_method<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    let method = call_method_name(source, node)?;
    if !DANGEROUS.contains(&method) {
        return None;
    }
    let recv = call_receiver(node)?;
    if recv.kind() != "constant" || node_bytes(source, recv) != b"IO" {
        return None;
    }
    if argument_nodes(node)
        .first()
        .is_some_and(|a| arg_starts_with_pipe(source, *a))
    {
        return None;
    }
    Some(method)
}

fn arg_starts_with_pipe(source: &SourceFile, node: Node<'_>) -> bool {
    let kind = node.kind();
    if kind != "string" && kind != "string_content" && !kind.contains("string") {
        return false;
    }
    string_inner(node_bytes(source, node)).starts_with(b"|")
}

fn string_inner(bytes: &[u8]) -> &[u8] {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.trim().trim_matches(|c| c == '"' || c == '\'').trim().as_bytes(),
        Err(_) => b"",
    }
}
