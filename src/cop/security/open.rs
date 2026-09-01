//! Security/Open — Kernel#open / URI.open without safe string literal.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct Open;

impl Cop for Open {
    fn name(&self) -> &'static str {
        "Security/Open"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "command_call"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"open"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if call_method_name(source, node) != Some(b"open") {
            return;
        }
        let Some(msg) = open_offense_msg(source, node) else {
            return;
        };
        let meth_node = node.child_by_field_name("method").unwrap_or(node);
        let (line, column) = source.offset_to_line_col(meth_node.start_byte());
        diagnostics.push(self.diagnostic(source, line, column, msg));
    }
}

fn open_offense_msg(source: &SourceFile, node: Node<'_>) -> Option<String> {
    let receiver_label = uri_receiver(source, node)?;
    let args = argument_nodes(node);
    if args.is_empty() && !has_block_arg(node) {
        return None;
    }
    if args.first().is_some_and(|a| is_safe_arg(source, *a)) {
        return None;
    }
    Some(match receiver_label {
        Some(name) => {
            let receiver = std::str::from_utf8(name).unwrap_or("URI");
            format!("The use of `{receiver}.open` is a serious security risk.")
        }
        None => "The use of `Kernel#open` is a serious security risk.".to_string(),
    })
}

/// `None` = not an open we care about; `Some(None)` = Kernel#open; `Some(Some(URI))` = URI.open.
fn uri_receiver<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<Option<&'a [u8]>> {
    match call_receiver(node) {
        None => Some(None),
        Some(recv) => {
            let src = node_bytes(source, recv);
            if src == b"URI" || src == b"::URI" {
                Some(Some(src))
            } else {
                None
            }
        }
    }
}

fn has_block_arg(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.named_children(&mut cur).any(|c| c.kind() == "block_argument")
}

fn is_safe_arg(source: &SourceFile, node: Node<'_>) -> bool {
    match node.kind() {
        "string" | "simple_symbol" => {
            let content = strip_quotes(node_bytes(source, node));
            !content.is_empty() && !content.starts_with(b"|")
        }
        "global_variable" => false,
        "binary" => {
            let mut cur = node.walk();
            node.named_children(&mut cur)
                .next()
                .is_some_and(|left| is_safe_arg(source, left))
        }
        _ => node_bytes(source, node) == b"__FILE__",
    }
}

fn strip_quotes(bytes: &[u8]) -> &[u8] {
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &bytes[1..bytes.len() - 1]
    } else {
        bytes
    }
}
