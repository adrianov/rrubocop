//! Security/JSONLoad — prefer JSON.parse over JSON.load/restore.

use tree_sitter::Node;

use crate::cop::shared::{
    argument_nodes, call_method_name, call_receiver, is_const_named, method_node, node_bytes,
    push_replace,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct JsonLoad;

impl Cop for JsonLoad {
    fn name(&self) -> &'static str {
        "Security/JSONLoad"
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
        let Some(method) = json_load_method(source, node) else {
            return;
        };
        let method_str = std::str::from_utf8(method).unwrap_or("load");
        let meth_node = method_node(node).unwrap_or(node);
        let (line, column) = source.offset_to_line_col(meth_node.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            format!("Prefer `JSON.parse` over `JSON.{method_str}`."),
        );
        if push_replace(
            &mut corrections,
            meth_node.start_byte(),
            meth_node.end_byte(),
            "parse",
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}

fn json_load_method<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    let method = call_method_name(source, node)?;
    if method != b"load" && method != b"restore" {
        return None;
    }
    let recv = call_receiver(node)?;
    if !is_const_named(source, recv, b"JSON") || has_create_additions(source, node) {
        return None;
    }
    Some(method)
}

fn has_create_additions(source: &SourceFile, node: Node<'_>) -> bool {
    for arg in argument_nodes(node) {
        if pair_key_is(source, arg, b"create_additions") {
            return true;
        }
        if arg.kind() == "hash" || arg.kind() == "pair" {
            let mut cur = arg.walk();
            for child in arg.named_children(&mut cur) {
                if pair_key_is(source, child, b"create_additions") {
                    return true;
                }
            }
        }
    }
    false
}

fn pair_key_is(source: &SourceFile, node: Node<'_>, key: &[u8]) -> bool {
    if node.kind() != "pair" {
        return false;
    }
    let Some(k) = node.child_by_field_name("key") else {
        return false;
    };
    let bytes = node_bytes(source, k);
    bytes == key || bytes.strip_prefix(b":") == Some(key)
}
