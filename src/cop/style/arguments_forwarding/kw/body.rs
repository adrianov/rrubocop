//! Which `**name` splats forward the keyword rest, and which reads keep the name.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

pub(super) struct Found<'a> {
    pub forwards: Vec<Node<'a>>,
    pub used: bool,
}

pub(super) fn scan<'a>(source: &'a SourceFile, body: Node<'a>, name: &[u8]) -> Found<'a> {
    let mut found = Found {
        forwards: Vec::new(),
        used: false,
    };
    let mut stack = vec![body];
    while let Some(node) = stack.pop() {
        if node.id() != body.id() && nested(node) {
            continue;
        }
        note(source, node, name, &mut found);
        push_named(&mut stack, node);
    }
    found
}

pub(super) fn in_block(node: Node<'_>) -> bool {
    let mut parent = node.parent();
    while let Some(current) = parent {
        if matches!(current.kind(), "block" | "do_block" | "lambda") {
            return true;
        }
        if matches!(current.kind(), "method" | "singleton_method") {
            return false;
        }
        parent = current.parent();
    }
    false
}

fn nested(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "method" | "singleton_method" | "class" | "module" | "singleton_class"
    )
}

fn note<'a>(source: &SourceFile, node: Node<'a>, name: &[u8], found: &mut Found<'a>) {
    if node.kind() == "hash_splat_argument" && splat_ident(source, node) == Some(name) {
        if call_arg(node) {
            found.forwards.push(node);
        }
    } else if shorthand(source, node, name) || ident_use(source, node, name) {
        found.used = true;
    }
}

fn splat_ident<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    let mut cur = node.walk();
    let mut kids = node.named_children(&mut cur);
    let only = kids.next()?;
    (kids.next().is_none() && only.kind() == "identifier").then(|| node_bytes(source, only))
}

fn call_arg(node: Node<'_>) -> bool {
    let mut parent = node.parent();
    while let Some(current) = parent {
        match current.kind() {
            "element_reference" => return outside_object(current, node),
            "argument_list" => return forwards_owner(current),
            "method" | "singleton_method" | "class" | "module" | "singleton_class" => return false,
            _ => parent = current.parent(),
        }
    }
    false
}

fn outside_object(reference: Node<'_>, node: Node<'_>) -> bool {
    reference
        .child_by_field_name("object")
        .is_none_or(|object| {
            node.start_byte() < object.start_byte() || node.end_byte() > object.end_byte()
        })
}

fn forwards_owner(list: Node<'_>) -> bool {
    list.parent().is_some_and(|owner| {
        matches!(
            owner.kind(),
            "call" | "command" | "command_call" | "yield" | "yield_command" | "element_reference"
        )
    })
}

fn push_named<'a>(stack: &mut Vec<Node<'a>>, node: Node<'a>) {
    let mut cur = node.walk();
    stack.extend(node.named_children(&mut cur));
}

fn shorthand(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    if node.kind() != "pair" || node.child_by_field_name("value").is_some() {
        return false;
    }
    node.child_by_field_name("key")
        .is_some_and(|key| key.kind() == "hash_key_symbol" && node_bytes(source, key) == name)
}

fn ident_use(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    node.kind() == "identifier"
        && node_bytes(source, node) == name
        && node.parent().is_some_and(|parent| !skipped(parent, node))
}

fn skipped(parent: Node<'_>, node: Node<'_>) -> bool {
    matches!(
        parent.kind(),
        "hash_splat_argument" | "block_parameters" | "lambda_parameters" | "method_parameters"
    ) || parent
        .child_by_field_name("method")
        .is_some_and(|method| method.id() == node.id())
}
