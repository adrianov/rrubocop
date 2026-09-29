//! Detect implicit `it`, `_1`, and a single named block argument.

use tree_sitter::Node;

use super::shadow::{body_assigns_it, enclosing_scope, outer_param, scope_assigns_it, walk_skip};
use crate::cop::shared::{method_node, node_bytes};
use crate::parse::source::SourceFile;

pub(super) fn host_node<'a>(block: Node<'a>) -> Node<'a> {
    match block.parent() {
        Some(p) if matches!(p.kind(), "call" | "command" | "command_call" | "lambda") => p,
        _ => block,
    }
}

pub(super) fn single_line(source: &SourceFile, node: Node<'_>) -> bool {
    source.offset_to_line_col(node.start_byte()).0
        == source
            .offset_to_line_col(node.end_byte().saturating_sub(1))
            .0
}

pub(super) fn explicit_params(block: Node<'_>) -> bool {
    if block.child_by_field_name("parameters").is_some() {
        return true;
    }
    block
        .parent()
        .is_some_and(|p| p.kind() == "lambda" && p.child_by_field_name("parameters").is_some())
}

pub(super) fn block_body(block: Node<'_>) -> Option<Node<'_>> {
    block.child_by_field_name("body")
}

fn is_method_name(node: Node<'_>) -> bool {
    node.parent()
        .and_then(method_node)
        .is_some_and(|m| m.id() == node.id())
}

pub(super) fn idents<'a>(source: &SourceFile, root: Node<'a>, name: &[u8]) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    walk_skip(root, None, |n| {
        if n.kind() == "identifier" && node_bytes(source, n) == name && !is_method_name(n) {
            out.push(n);
        }
    });
    out
}

fn numbered(name: &[u8]) -> Option<u32> {
    let rest = name.strip_prefix(b"_")?;
    if rest.is_empty() || rest[0] == b'0' || !rest.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(rest).ok()?.parse().ok()
}

fn push_numbered<'a>(source: &SourceFile, node: Node<'a>, nums: &mut Vec<(u32, Node<'a>)>) {
    if node.kind() != "identifier" || is_method_name(node) {
        return;
    }
    if let Some(v) = numbered(node_bytes(source, node)) {
        nums.push((v, node));
    }
}

fn only_first<'a>(nums: Vec<(u32, Node<'a>)>) -> Vec<Node<'a>> {
    if nums.iter().any(|(v, _)| *v > 1) {
        return Vec::new();
    }
    nums.into_iter()
        .filter(|(v, _)| *v == 1)
        .map(|(_, n)| n)
        .collect()
}

pub(super) fn num_idents<'a>(source: &SourceFile, root: Node<'a>) -> Vec<Node<'a>> {
    let mut nums = Vec::new();
    walk_skip(root, None, |n| push_numbered(source, n, &mut nums));
    only_first(nums)
}

pub(super) fn is_it_block(source: &SourceFile, block: Node<'_>) -> bool {
    if explicit_params(block) {
        return false;
    }
    let Some(body) = block_body(block) else {
        return false;
    };
    if body_assigns_it(source, body) || outer_param(source, block) {
        return false;
    }
    if scope_assigns_it(source, enclosing_scope(block), block) {
        return false;
    }
    !idents(source, body, b"it").is_empty()
}

pub(super) fn plain_arg<'a>(source: &'a SourceFile, block: Node<'_>) -> Option<&'a [u8]> {
    let params = block.child_by_field_name("parameters")?;
    let mut cur = params.walk();
    let kids: Vec<_> = params.named_children(&mut cur).collect();
    (kids.len() == 1 && kids[0].kind() == "identifier").then(|| node_bytes(source, kids[0]))
}
