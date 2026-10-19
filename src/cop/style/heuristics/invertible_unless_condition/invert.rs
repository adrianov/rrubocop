//! RuboCop `preferred_condition` + `autocorrect` for an invertible condition:
//! the inverted source text and the in-place edits that produce it.

use tree_sitter::Node;

use super::{Inverses, operator, text};
use crate::cop::shared::{Edit, field_text, named_kids};
use crate::parse::source::SourceFile;

pub(super) fn invert(
    node: Node<'_>,
    source: &SourceFile,
    inv: &Inverses<'_>,
    edits: &mut Vec<Edit>,
) -> String {
    match node.kind() {
        "parenthesized_statements" => {
            let inner = node
                .named_child(0)
                .map(|first| invert(first, source, inv, edits));
            format!("({})", inner.unwrap_or_default())
        }
        "unary" => {
            replace(node.child_by_field_name("operator"), "", edits);
            field_text(source, node, "operand").to_string()
        }
        "binary" => match logical_inverse(operator(node, source)) {
            Some(new_op) => invert_logical(node, new_op, source, inv, edits),
            None => invert_send(node, source, inv, edits),
        },
        _ => invert_send(node, source, inv, edits),
    }
}

fn logical_inverse(op: &str) -> Option<&'static str> {
    match op {
        "&&" => Some("||"),
        "||" => Some("&&"),
        "and" => Some("or"),
        "or" => Some("and"),
        _ => None,
    }
}

fn invert_logical(
    node: Node<'_>,
    new_op: &str,
    source: &SourceFile,
    inv: &Inverses<'_>,
    edits: &mut Vec<Edit>,
) -> String {
    replace(node.child_by_field_name("operator"), new_op, edits);
    let mut side = |field: &str| {
        node.child_by_field_name(field)
            .map(|n| invert(n, source, inv, edits))
            .unwrap_or_default()
    };
    let left = side("left");
    format!("{left} {new_op} {}", side("right"))
}

/// A method call or an operator comparison (both `send` in RuboCop):
/// rename the method to its inverse, or drop a `!`.
fn invert_send(
    node: Node<'_>,
    source: &SourceFile,
    inv: &Inverses<'_>,
    edits: &mut Vec<Edit>,
) -> String {
    let binary = node.kind() == "binary";
    let method = node.child_by_field_name(if binary { "operator" } else { "method" });
    let name = method.map_or("", |m| text(m, source));
    let new_name = match name {
        "!" | "not" => "",
        _ => inv.get(name).copied().unwrap_or(name),
    };
    replace(method, new_name, edits);
    if binary {
        let (left, right) = (
            field_text(source, node, "left"),
            field_text(source, node, "right"),
        );
        format!("{left} {new_name} {right}")
    } else {
        call_text(node, new_name, source)
    }
}

/// RuboCop `preferred_send_condition` for a call renamed to `name`
/// (empty `name` is a removed `!`, leaving the receiver).
fn call_text(node: Node<'_>, name: &str, source: &SourceFile) -> String {
    let receiver = field_text(source, node, "receiver");
    if name.is_empty() {
        return receiver.to_string();
    }
    let dotted = if receiver.is_empty() {
        String::new()
    } else {
        format!("{receiver}.")
    };
    format!("{dotted}{name}{}", args_text(node, source))
}

/// Arguments re-joined with `, `, keeping parentheses or the bare-space form.
fn args_text(node: Node<'_>, source: &SourceFile) -> String {
    let Some(args) = node.child_by_field_name("arguments") else {
        return String::new();
    };
    let list: Vec<&str> = named_kids(args).iter().map(|a| text(*a, source)).collect();
    if text(args, source).starts_with('(') {
        format!("({})", list.join(", "))
    } else {
        format!(" {}", list.join(", "))
    }
}

fn replace(token: Option<Node<'_>>, with: &str, edits: &mut Vec<Edit>) {
    if let Some(t) = token {
        edits.push((t.start_byte(), t.end_byte(), with.to_string()));
    }
}
