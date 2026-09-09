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
    if let Some(body) = node.child_by_field_name("body") {
        push_value(out, body);
    } else {
        let mut cur = node.walk();
        if let Some(cons) = node.named_children(&mut cur).next() {
            push_value(out, cons);
        }
    }
    out.push(Ret::Nil);
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

fn take_if_alternative<'a>(out: &mut Vec<Ret<'a>>, alt: Node<'a>) -> bool {
    if alt.kind() == "else" {
        push_else_branch(out, alt);
        return true;
    }
    push_value(out, alt);
    false
}

fn expand_if_chain<'a>(out: &mut Vec<Ret<'a>>, node: Node<'a>) {
    // RuboCop: nil for missing else only on plain `if/end` (no subsequent).
    let top_has_subsequent = node.child_by_field_name("alternative").is_some();
    let mut cur = Some(node);
    let mut saw_else = false;
    while let Some(n) = cur.take() {
        if let Some(cons) = n.child_by_field_name("consequence") {
            push_value(out, cons);
        }
        let Some(alt) = n.child_by_field_name("alternative") else {
            break;
        };
        if alt.kind() == "elsif" {
            cur = Some(alt);
            continue;
        }
        saw_else = take_if_alternative(out, alt);
        break;
    }
    if !saw_else && !top_has_subsequent {
        out.push(Ret::Nil);
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
    let mut has_else = false;
    let mut cur = node.walk();
    for child in node.named_children(&mut cur) {
        match child.kind() {
            "when" | "in" => push_ret(out, when_body(child)),
            "else" => {
                has_else = true;
                push_else_branch(out, child);
            }
            _ => {}
        }
    }
    if !has_else {
        out.push(Ret::Nil);
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
