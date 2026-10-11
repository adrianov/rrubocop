//! Where `&name` is inserted into a parameter list, an argument list, or bare `super`.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

pub(super) fn list_edit(
    source: &SourceFile,
    list: Node<'_>,
    name: &str,
) -> Option<(usize, usize, String)> {
    match last_named(list) {
        Some(arg) => arg_edit(source, arg, name),
        None => empty_parens(source, list, name),
    }
}

fn last_named(list: Node<'_>) -> Option<Node<'_>> {
    let mut cur = list.walk();
    list.named_children(&mut cur).last()
}

fn empty_parens(source: &SourceFile, list: Node<'_>, name: &str) -> Option<(usize, usize, String)> {
    source
        .as_bytes()
        .get(list.start_byte())
        .eq(&Some(&b'('))
        .then(|| (list.start_byte(), list.end_byte(), format!("(&{name})")))
}

fn arg_edit(source: &SourceFile, arg: Node<'_>, name: &str) -> Option<(usize, usize, String)> {
    if matches!(arg.kind(), "block_argument" | "block_parameter") {
        return None;
    }
    let end = arg.end_byte();
    if source.as_bytes().get(end) == Some(&b',') {
        Some((end + 1, end + 1, format!(" &{name}")))
    } else {
        Some((end, end, format!(", &{name}")))
    }
}

/// Bare `super` forwards the evaluated local. `b = 1` becomes `b`, `j: 1` becomes `j: j`.
fn push_forwarded(source: &SourceFile, param: Node<'_>, parts: &mut Vec<String>) {
    if param.kind() == "destructured_parameter" {
        let mut cur = param.walk();
        for child in param.named_children(&mut cur) {
            push_forwarded(source, child, parts);
        }
        return;
    }
    parts.push(forwarded(source, param));
}

fn forwarded(source: &SourceFile, param: Node<'_>) -> String {
    match param.kind() {
        "optional_parameter" => param_name(source, param),
        "keyword_parameter" if param.child_by_field_name("value").is_some() => {
            let name = param_name(source, param);
            format!("{name}: {name}")
        }
        _ => String::from_utf8_lossy(node_bytes(source, param)).into_owned(),
    }
}

fn param_name(source: &SourceFile, param: Node<'_>) -> String {
    param
        .child_by_field_name("name")
        .map(|n| String::from_utf8_lossy(node_bytes(source, n)).into_owned())
        .unwrap_or_default()
}

pub(super) fn zsuper_args(source: &SourceFile, method: Node<'_>, name: &str) -> String {
    let mut parts = Vec::new();
    if let Some(params) = method.child_by_field_name("parameters") {
        let mut cur = params.walk();
        for param in params.named_children(&mut cur) {
            if param.kind() != "block_parameter" {
                push_forwarded(source, param, &mut parts);
            }
        }
    }
    parts.push(format!("&{name}"));
    parts.join(", ")
}

pub(super) fn is_zsuper(call: Node<'_>) -> bool {
    call.child_by_field_name("arguments").is_none()
        && call
            .child_by_field_name("method")
            .is_some_and(|m| m.kind() == "super")
}
