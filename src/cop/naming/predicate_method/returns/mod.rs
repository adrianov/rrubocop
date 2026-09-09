//! Return-value collection for Naming/PredicateMethod.

mod expand;

use tree_sitter::Node;

use crate::parse::source::SourceFile;

use expand::{expand_and_or, expand_and_or_binary, push_value};

/// One analyzed return (real AST node or RuboCop-synthesized `nil`).
#[derive(Clone, Copy, Debug)]
pub(super) enum Ret<'a> {
    Node(Node<'a>),
    Nil,
}

pub(super) fn push_ret<'a>(out: &mut Vec<Ret<'a>>, ret: Ret<'a>) {
    match ret {
        Ret::Nil => out.push(Ret::Nil),
        Ret::Node(n) => push_value(out, n),
    }
}

fn is_comment(node: Node<'_>) -> bool {
    node.kind() == "comment"
}

/// Last non-comment named child (comments are named in tree-sitter-ruby).
pub(super) fn last_significant<'a>(body: Node<'a>) -> Option<Node<'a>> {
    let mut cur = body.walk();
    body.named_children(&mut cur).filter(|n| !is_comment(*n)).last()
}

/// Unwrap `then` / `else` / `body_statement` wrappers to the value expression.
pub(super) fn unwrap_branch_body(node: Node<'_>) -> Ret<'_> {
    match node.kind() {
        "then" | "else" | "body_statement" => match last_significant(node) {
            Some(inner) => unwrap_branch_body(inner),
            None => Ret::Nil,
        },
        _ => Ret::Node(node),
    }
}

pub(super) fn extract_return_arg(node: Node<'_>) -> Option<Ret<'_>> {
    if node.kind() != "return" {
        return Some(Ret::Node(node));
    }
    let Some(arg) = node.child_by_field_name("arguments").or_else(|| {
        let mut cur = node.walk();
        node.named_children(&mut cur).next()
    }) else {
        return Some(Ret::Nil);
    };
    match unwrap_arg_list(arg) {
        None => Some(Ret::Nil),
        Some(n) => Some(Ret::Node(n)),
    }
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

fn needs_renormalize(ret: Ret<'_>) -> bool {
    match ret {
        Ret::Nil => false,
        Ret::Node(node) => matches!(
            node.kind(),
            "if" | "unless"
                | "if_modifier"
                | "unless_modifier"
                | "and"
                | "or"
                | "conditional"
                | "case"
                | "elsif"
                | "else"
                | "then"
                | "body_statement"
                | "return"
        ),
    }
}

fn normalize_one<'a>(source: &SourceFile, n: Node<'a>, out: &mut Vec<Ret<'a>>) {
    match n.kind() {
        "and" | "or" => expand_and_or(out, n),
        "binary" => expand_and_or_binary(source, n, out),
        "if" | "unless" | "if_modifier" | "unless_modifier" | "case" | "conditional" | "elsif"
        | "else" | "then" | "body_statement" | "return" => {
            push_value(out, n);
        }
        _ => out.push(Ret::Node(n)),
    }
}

pub(super) fn normalize_values<'a>(source: &SourceFile, values: Vec<Ret<'a>>) -> Vec<Ret<'a>> {
    let mut out = Vec::new();
    for v in values {
        match v {
            Ret::Nil => out.push(Ret::Nil),
            Ret::Node(n) => normalize_one(source, n, &mut out),
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

fn push_implicit_body<'a>(body: Node<'a>, out: &mut Vec<Ret<'a>>) {
    if body.kind() != "body_statement" {
        push_value(out, body);
    } else if let Some(last) = last_significant(body) {
        push_value(out, last);
    }
}

fn collect_explicit_returns<'a>(body: Node<'a>, out: &mut Vec<Ret<'a>>) {
    let mut stack = vec![body];
    while let Some(n) = stack.pop() {
        if n.kind() == "return" {
            if let Some(v) = extract_return_arg(n) {
                push_ret(out, v);
            }
        }
        let mut cur = n.walk();
        for child in n.named_children(&mut cur) {
            if !nested_return_scope(child.kind()) {
                stack.push(child);
            }
        }
    }
}

/// Collect implicit (last expr) and explicit `return` values from a method body.
pub(super) fn collect_returns<'a>(body: Node<'a>) -> Vec<Ret<'a>> {
    let mut out = Vec::new();
    push_implicit_body(body, &mut out);
    collect_explicit_returns(body, &mut out);
    out
}

