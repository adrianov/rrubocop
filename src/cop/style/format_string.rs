//! Style/FormatString — prefer format/sprintf/%.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FormatString;

impl Cop for FormatString {
    fn name(&self) -> &'static str {
        "Style/FormatString"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "binary"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "format");
        let Some((msg, at)) = format_offense(source, node, style) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(at);
        diagnostics.push(self.diagnostic(source, line, col, msg));
    }
}

fn format_offense(source: &SourceFile, node: Node<'_>, style: &str) -> Option<(String, usize)> {
    if node.kind() == "binary" {
        return percent_binary(source, node, style);
    }
    let method = call_method_name(source, node)?;
    match method {
        b"format" | b"sprintf" => {
            let current = std::str::from_utf8(method).unwrap_or("format");
            if current == style {
                return None;
            }
            Some((
                format!("Favor `{style}` over `{current}`."),
                node.child_by_field_name("method")
                    .map(|m| m.start_byte())
                    .unwrap_or(node.start_byte()),
            ))
        }
        b"%" => percent_send(source, node, style),
        _ => None,
    }
}

/// RuboCop: `(send {str dstr} :% ...)` or `(send !nil? :% {array hash})`.
fn percent_send(_source: &SourceFile, node: Node<'_>, style: &str) -> Option<(String, usize)> {
    if style == "percent" {
        return None;
    }
    let recv = call_receiver(node)?;
    let args = crate::cop::shared::argument_nodes(node);
    let ok = is_string_like(recv)
        || args
            .first()
            .is_some_and(|a| matches!(a.kind(), "array" | "hash"));
    ok.then(|| {
        (
            format!("Favor `{style}` over `String#%`."),
            node.child_by_field_name("method")
                .map(|m| m.start_byte())
                .unwrap_or(node.start_byte()),
        )
    })
}

fn percent_binary(source: &SourceFile, node: Node<'_>, style: &str) -> Option<(String, usize)> {
    if style == "percent" {
        return None;
    }
    let mut cur = node.walk();
    let kids: Vec<_> = node.children(&mut cur).collect();
    if kids.len() < 3 || node_bytes(source, kids[1]) != b"%" {
        return None;
    }
    let left = kids[0];
    let right = kids[2];
    let ok = is_string_like(left) || matches!(right.kind(), "array" | "hash");
    ok.then(|| {
        (
            format!("Favor `{style}` over `String#%`."),
            kids[1].start_byte(),
        )
    })
}

fn is_string_like(n: Node<'_>) -> bool {
    matches!(n.kind(), "string" | "chained_string" | "heredoc_beginning")
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(FormatString, "cops/style/format_string");
}
