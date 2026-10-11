//! Style/ExplicitBlockArgument — a block whose body is only `yield` of its args.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_explicit_block_argument(
    source: &SourceFile,
    node: Node<'_>,
    _config: &CopConfig,
) -> bool {
    if !matches!(node.kind(), "block" | "do_block") {
        return false;
    }
    let Some(yield_node) = sole_yield(node) else {
        return false;
    };
    if !in_method(node) {
        return false;
    }
    match (param_names(source, node), yield_names(source, yield_node)) {
        (Some(params), Some(yielded)) => names_match(&params, &yielded),
        _ => false,
    }
}

fn sole_yield(block: Node<'_>) -> Option<Node<'_>> {
    let body = block.child_by_field_name("body")?;
    if body.kind() == "yield" {
        return Some(body);
    }
    let mut cur = body.walk();
    let mut kids = body
        .named_children(&mut cur)
        .filter(|n| !matches!(n.kind(), "comment" | "empty_statement"));
    let first = kids.next()?;
    if kids.next().is_some() || first.kind() != "yield" {
        return None;
    }
    Some(first)
}

fn in_method(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(n) = p {
        if matches!(n.kind(), "method" | "singleton_method") {
            return true;
        }
        p = n.parent();
    }
    false
}

fn param_names<'a>(source: &'a SourceFile, block: Node<'_>) -> Option<Vec<&'a [u8]>> {
    match block.child_by_field_name("parameters") {
        Some(params) => idents(source, params),
        None => Some(Vec::new()),
    }
}

fn yield_names<'a>(source: &'a SourceFile, yield_node: Node<'_>) -> Option<Vec<&'a [u8]>> {
    let mut cur = yield_node.walk();
    match yield_node
        .named_children(&mut cur)
        .find(|n| n.kind() == "argument_list")
    {
        Some(list) => idents(source, list),
        None => Some(Vec::new()),
    }
}

fn idents<'a>(source: &'a SourceFile, list: Node<'_>) -> Option<Vec<&'a [u8]>> {
    let mut cur = list.walk();
    let mut names = Vec::new();
    for child in list.named_children(&mut cur) {
        if child.kind() != "identifier" {
            return None;
        }
        names.push(node_bytes(source, child));
    }
    Some(names)
}

/// Ruby `zip` pads the shorter list with nil, so lengths must match too.
fn names_match(block_args: &[&[u8]], yield_args: &[&[u8]]) -> bool {
    (0..block_args.len().max(yield_args.len())).all(|i| block_args.get(i) == yield_args.get(i))
}
