use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, node_bytes, node_text};
use crate::parse::source::SourceFile;

pub(super) fn gem_name(source: &SourceFile, node: Node<'_>) -> Option<String> {
    let first = argument_nodes(node).into_iter().next()?;
    gem_name_from_node(source, first)
}

fn gem_name_from_node(source: &SourceFile, mut node: Node<'_>) -> Option<String> {
    loop {
        match node.kind() {
            "string" => {
                let bytes = node_bytes(source, node);
                return Some(String::from_utf8_lossy(strip_quotes(bytes)).into_owned());
            }
            "call" | "method_call" | "command_call" => {
                node = node.child_by_field_name("receiver")?;
            }
            _ => {
                let text = node_text(source, node);
                return extract_gem_name_text(&text);
            }
        }
    }
}

fn extract_gem_name_text(text: &str) -> Option<String> {
    let s = text.trim();
    if s.starts_with('\'') || s.starts_with('"') {
        let quote = s.as_bytes()[0];
        let rest = &s[1..];
        let end = rest.find(|c: char| c as u8 == quote)?;
        return Some(rest[..end].to_string());
    }
    parse_percent_string(s).map(|(name, _)| name)
}

fn parse_percent_string(s: &str) -> Option<(String, usize)> {
    let rest = s.strip_prefix('%')?;
    let (body, base) = percent_body(rest)?;
    let (close, open_len) = percent_delim(body)?;
    let inner = &body[open_len..];
    let end = inner.find(close)?;
    Some((inner[..end].to_string(), base + open_len + end + 1))
}

fn percent_body(rest: &str) -> Option<(&str, usize)> {
    match rest.as_bytes().first()? {
        b'q' | b'Q' => Some((&rest[1..], 2)),
        _ => Some((rest, 1)),
    }
}

fn percent_delim(body: &str) -> Option<(char, usize)> {
    match body.as_bytes().first()? {
        b'<' => Some(('>', 1)),
        b'(' => Some((')', 1)),
        b'[' => Some((']', 1)),
        b'{' => Some(('}', 1)),
        _ => None,
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
