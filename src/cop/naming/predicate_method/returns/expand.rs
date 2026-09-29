//! Expand conditionals / and-or into individual return values.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

use super::{
    extract_return_arg, last_significant, push_ret, unwrap_branch_body, Ret,
};

pub(super) fn expand_and_or<'a>(out: &mut Vec<Ret<'a>>, node: Node<'a>) {
    let mut cur = node.walk();
    let kids: Vec<_> = node.named_children(&mut cur).collect();
    if kids.len() == 2 {
        push_value(out, kids[0]);
        push_value(out, kids[1]);
    } else {
        out.push(Ret::Node(node));
    }
}

fn expand_modifier<'a>(out: &mut Vec<Ret<'a>>, node: Node<'a>) {
    // RuboCop `IfNode#branches` is only the written body — no nil for `expr if cond`.
    if let Some(body) = node.child_by_field_name("body") {
        push_value(out, body);
        return;
    }
    let mut cur = node.walk();
    if let Some(cons) = node.named_children(&mut cur).next() {
        push_value(out, cons);
    }
}

fn expand_conditional_fields<'a>(out: &mut Vec<Ret<'a>>, node: Node<'a>) {
    if let Some(cons) = node.child_by_field_name("consequence") {
        push_value(out, cons);
    }
    match node.child_by_field_name("alternative") {
        Some(alt) => push_value(out, alt),
        None => out.push(Ret::Nil),
    }
}

fn push_else_branch<'a>(out: &mut Vec<Ret<'a>>, alt: Node<'a>) {
    push_ret(out, unwrap_branch_body(alt));
}

fn push_if_branch<'a>(out: &mut Vec<Ret<'a>>, node: Node<'a>) {
    match node.child_by_field_name("consequence") {
        Some(cons) => push_value(out, cons),
        None => out.push(Ret::Nil),
    }
}

fn expand_if_chain<'a>(out: &mut Vec<Ret<'a>>, node: Node<'a>) {
    // `IfNode#branches` omits a missing else. An empty branch (`if x; else`) is nil.
    let mut cur = Some(node);
    while let Some(n) = cur.take() {
        push_if_branch(out, n);
        let Some(alt) = n.child_by_field_name("alternative") else {
            break;
        };
        if alt.kind() == "elsif" {
            cur = Some(alt);
            continue;
        }
        push_else_branch(out, alt);
        break;
    }
}

fn when_body<'a>(child: Node<'a>) -> Ret<'a> {
    if let Some(body) = child.child_by_field_name("body") {
        return Ret::Node(body);
    }
    last_significant(child)
        .map(Ret::Node)
        .unwrap_or(Ret::Nil)
}

fn expand_case<'a>(out: &mut Vec<Ret<'a>>, node: Node<'a>) {
    // `CaseNode#branches` omits a missing else (no implicit nil).
    let mut cur = node.walk();
    for child in node.named_children(&mut cur) {
        match child.kind() {
            "when" | "in" | "in_clause" => push_ret(out, when_body(child)),
            "else" => push_else_branch(out, child),
            _ => {}
        }
    }
}

fn expand_conditional<'a>(out: &mut Vec<Ret<'a>>, node: Node<'a>) {
    match node.kind() {
        "if_modifier" | "unless_modifier" => expand_modifier(out, node),
        "conditional" => expand_conditional_fields(out, node),
        "if" | "unless" | "elsif" => expand_if_chain(out, node),
        "case" => expand_case(out, node),
        "else" => push_else_branch(out, node),
        _ => out.push(Ret::Node(node)),
    }
}

pub(super) fn push_value<'a>(out: &mut Vec<Ret<'a>>, node: Node<'a>) {
    match node.kind() {
        "if" | "unless" | "if_modifier" | "unless_modifier" | "case" | "conditional" | "elsif"
        | "else" => expand_conditional(out, node),
        "and" | "or" => expand_and_or(out, node),
        "then" | "body_statement" => push_ret(out, unwrap_branch_body(node)),
        "return" => {
            if let Some(v) = extract_return_arg(node) {
                push_ret(out, v);
            }
        }
        _ => out.push(Ret::Node(node)),
    }
}

fn is_and_or_op(source: &SourceFile, node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.children(&mut cur)
        .any(|c| !c.is_named() && matches!(node_bytes(source, c), b"&&" | b"||"))
}

pub(super) fn expand_and_or_binary<'a>(source: &SourceFile, node: Node<'a>, out: &mut Vec<Ret<'a>>) {
    if node.kind() != "binary" || !is_and_or_op(source, node) {
        out.push(Ret::Node(node));
        return;
    }
    let mut cur = node.walk();
    let kids: Vec<_> = node.named_children(&mut cur).collect();
    if kids.len() != 2 {
        out.push(Ret::Node(node));
        return;
    }
    expand_and_or_binary(source, kids[0], out);
    expand_and_or_binary(source, kids[1], out);
}
