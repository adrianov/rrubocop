//! Return-value collection for Naming/PredicateMethod.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

fn extract_return_arg(node: Node<'_>) -> Option<Node<'_>> {
    if node.kind() != "return" {
        return Some(node);
    }
    let arg = node.child_by_field_name("arguments").or_else(|| {
        let mut cur = node.walk();
        node.named_children(&mut cur).next()
    })?;
    unwrap_arg_list(arg)
}

fn unwrap_arg_list(arg: Node<'_>) -> Option<Node<'_>> {
    if arg.kind() != "argument_list" {
        return Some(arg);
    }
    let mut cur = arg.walk();
    let named: Vec<_> = arg.named_children(&mut cur).collect();
    match named.len() {
        0 => None,
        1 => Some(named[0]),
        _ => Some(arg),
    }
}

fn last_body_value(body: Node<'_>) -> Option<Node<'_>> {
    let mut cur = body.walk();
    body.named_children(&mut cur).last()
}

fn expand_and_or<'a>(out: &mut Vec<Node<'a>>, node: Node<'a>) {
    let mut cur = node.walk();
    let kids: Vec<_> = node.named_children(&mut cur).collect();
    if kids.len() == 2 {
        push_value(out, kids[0]);
        push_value(out, kids[1]);
    } else {
        out.push(node);
    }
}

fn expand_modifier<'a>(out: &mut Vec<Node<'a>>, node: Node<'a>) {
    let mut cur = node.walk();
    if let Some(cons) = node.named_children(&mut cur).next() {
        push_value(out, cons);
    }
}

fn expand_ternary<'a>(out: &mut Vec<Node<'a>>, node: Node<'a>) {
    let mut cur = node.walk();
    let kids: Vec<_> = node.named_children(&mut cur).collect();
    if kids.len() >= 3 {
        push_value(out, kids[1]);
        push_value(out, kids[2]);
    }
}

fn expand_if_like<'a>(out: &mut Vec<Node<'a>>, node: Node<'a>) {
    if let Some(cons) = node.child_by_field_name("consequence") {
        push_value(out, cons);
    }
    if let Some(alt) = node.child_by_field_name("alternative") {
        push_value(out, alt);
    }
}

fn expand_conditional<'a>(out: &mut Vec<Node<'a>>, node: Node<'a>) {
    match node.kind() {
        "if_modifier" | "unless_modifier" => expand_modifier(out, node),
        "ternary" => expand_ternary(out, node),
        _ => expand_if_like(out, node),
    }
}

fn push_value<'a>(out: &mut Vec<Node<'a>>, node: Node<'a>) {
    match node.kind() {
        "if" | "unless" | "if_modifier" | "unless_modifier" | "case" | "ternary" => {
            expand_conditional(out, node);
        }
        "and" | "or" => expand_and_or(out, node),
        _ => out.push(node),
    }
}

fn is_and_or_op(source: &SourceFile, node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.children(&mut cur)
        .any(|c| !c.is_named() && matches!(node_bytes(source, c), b"&&" | b"||"))
}

fn expand_and_or_binary<'a>(source: &SourceFile, node: Node<'a>, out: &mut Vec<Node<'a>>) {
    if node.kind() != "binary" || !is_and_or_op(source, node) {
        out.push(node);
        return;
    }
    let mut cur = node.walk();
    let kids: Vec<_> = node.named_children(&mut cur).collect();
    if kids.len() != 2 {
        out.push(node);
        return;
    }
    expand_and_or_binary(source, kids[0], out);
    expand_and_or_binary(source, kids[1], out);
}

fn needs_renormalize(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "if" | "unless" | "if_modifier" | "unless_modifier" | "and" | "or" | "ternary"
    )
}

pub(super) fn normalize_values<'a>(source: &SourceFile, values: Vec<Node<'a>>) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    for v in values {
        match v.kind() {
            "and" | "or" => expand_and_or(&mut out, v),
            "binary" => expand_and_or_binary(source, v, &mut out),
            "if" | "unless" | "if_modifier" | "unless_modifier" | "case" | "ternary" => {
                expand_conditional(&mut out, v);
            }
            _ => out.push(v),
        }
    }
    if out.iter().any(|n| needs_renormalize(*n)) {
        return normalize_values(source, out);
    }
    out
}

fn nested_return_scope(kind: &str) -> bool {
    matches!(kind, "method" | "singleton_method" | "lambda")
}

pub(super) fn collect_returns<'a>(body: Node<'a>) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    if let Some(last) = last_body_value(body) {
        push_value(&mut out, last);
    }
    let mut stack = vec![body];
    while let Some(n) = stack.pop() {
        if n.kind() == "return" {
            if let Some(v) = extract_return_arg(n) {
                push_value(&mut out, v);
            }
        }
        let mut cur = n.walk();
        for child in n.named_children(&mut cur) {
            // Nested defs / stabby lambdas have their own return semantics.
            if nested_return_scope(child.kind()) {
                continue;
            }
            stack.push(child);
        }
    }
    out
}
