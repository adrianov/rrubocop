//! Expression / do-line anchors for block `end` alignment.

use tree_sitter::Node;

use super::opener::first_non_ws_offset;
use crate::cop::shared;
use crate::parse::source::SourceFile;

pub(super) fn block_call_node<'a>(block: Node<'a>) -> Option<Node<'a>> {
    let parent = block.parent()?;
    if !matches!(parent.kind(), "call" | "command" | "command_call") {
        return None;
    }
    parent
        .child_by_field_name("block")
        .filter(|b| b.id() == block.id())
        .map(|_| parent)
}

fn inside_parentheses(source: &SourceFile, call: Node<'_>, pos: usize) -> bool {
    let bytes = source.as_bytes();
    let start = call.start_byte();
    if pos <= start {
        return false;
    }
    let mut opens = 0i32;
    let mut closes = 0i32;
    for &b in &bytes[start..pos.min(bytes.len())] {
        match b {
            b'(' | b'[' => opens += 1,
            b')' | b']' => closes += 1,
            _ => {}
        }
    }
    opens > closes
}

fn do_line_begins_inside_argument(source: &SourceFile, call: Node<'_>, opener_off: usize) -> bool {
    let (line, _) = source.offset_to_line_col(opener_off);
    let Some(first_char_off) = first_non_ws_offset(source, line) else {
        return false;
    };
    if !inside_parentheses(source, call, first_char_off) {
        return false;
    }
    let Some(args) = call.child_by_field_name("arguments") else {
        return false;
    };
    let mut cur = args.walk();
    args.named_children(&mut cur).any(|arg| {
        arg.start_byte() <= first_char_off && first_char_off < arg.end_byte()
    })
}

fn method_selector_offset(call: Node<'_>) -> usize {
    call.child_by_field_name("method")
        .map(|m| m.start_byte())
        .unwrap_or_else(|| call.start_byte())
}

/// When `do`/`{` sits on a multiline argument continuation line, anchor on the call.
pub(super) fn do_line_anchor_offset(source: &SourceFile, block: Node<'_>, opener: Node<'_>) -> usize {
    let Some(call) = block_call_node(block) else {
        return opener.start_byte();
    };
    if do_line_begins_inside_argument(source, call, opener.start_byte()) {
        method_selector_offset(call)
    } else {
        opener.start_byte()
    }
}

fn call_chain_root(mut call: Node<'_>) -> Node<'_> {
    while let Some(recv) = call.child_by_field_name("receiver") {
        if !matches!(recv.kind(), "call" | "command" | "command_call") {
            break;
        }
        call = recv;
    }
    call
}

/// RuboCop block nodes span the full send chain; tree-sitter `do_block` starts at `do`.
pub(super) fn block_expression_start<'a>(block: Node<'a>) -> Node<'a> {
    block_call_node(block)
        .map(call_chain_root)
        .unwrap_or(block)
}

pub(super) fn block_line_indent(source: &SourceFile, node: Node<'_>) -> usize {
    let start = block_expression_start(node);
    shared::line_indent(source, start.start_byte())
}

pub(super) fn expression_start_col(source: &SourceFile, block: Node<'_>) -> usize {
    expression_start_from_ancestors(source, block_expression_start(block))
}

pub(super) fn do_line_begin_col(source: &SourceFile, opener: Node<'_>) -> usize {
    let (line, _) = source.offset_to_line_col(opener.start_byte());
    first_non_ws_offset(source, line)
        .map(|off| source.offset_to_line_col(off).1)
        .unwrap_or_else(|| shared::line_indent(source, opener.start_byte()))
}

fn same_line(source: &SourceFile, a: Node<'_>, b: Node<'_>) -> bool {
    shared::node_line(source, a) == shared::node_line(source, b)
}

fn disqualified_parent(source: &SourceFile, parent: Node<'_>, current: Node<'_>) -> bool {
    !same_line(source, parent, current) && parent.kind() != "assignment"
}

fn is_assignment_parent(node: Node<'_>, source: &SourceFile) -> bool {
    match node.kind() {
        "assignment" | "operator_assignment" | "conditional_assignment" => true,
        "call" | "command" | "command_call" => shared::call_method_name(source, node)
            .is_some_and(|name| matches!(name, b"=" | b"+=" | b"-=" | b"*=" | b"/=")),
        _ => false,
    }
}

fn block_end_align_target(source: &SourceFile, parent: Node<'_>, current: Node<'_>) -> bool {
    if is_assignment_parent(parent, source) {
        return true;
    }
    match parent.kind() {
        "and" | "or" | "unary" | "splat_argument" => return true,
        "method" | "singleton_method" => return true,
        _ => {}
    }
    if !matches!(parent.kind(), "call" | "command" | "command_call") {
        return false;
    }
    if shared::call_method_name(source, parent) == Some(b"<<") {
        return true;
    }
    parent
        .child_by_field_name("receiver")
        .is_some_and(|recv| recv.id() == current.id())
        && shared::call_method_name(source, parent) != Some(b"[]")
}

fn expression_start_from_ancestors(source: &SourceFile, block: Node<'_>) -> usize {
    let mut current = block;
    let mut start_offset = block.start_byte();
    let initial = block;

    while let Some(parent) = current.parent() {
        if current.id() != initial.id()
            && parent
                .child_by_field_name("block")
                .is_some_and(|b| b.id() == current.id())
        {
            break;
        }
        if disqualified_parent(source, parent, current)
            || !block_end_align_target(source, parent, current)
        {
            break;
        }
        start_offset = parent.start_byte();
        current = parent;
    }

    source.offset_to_line_col(start_offset).1
}

pub(super) fn expression_start_for_line(source: &SourceFile, block: Node<'_>) -> usize {
    let mut start = block_expression_start(block);
    let mut col = expression_start_col(source, block);
    let mut current = start;
    while let Some(parent) = current.parent() {
        if !same_line(source, start, parent) {
            break;
        }
        col = shared::node_col(source, parent).min(shared::line_indent(source, parent.start_byte()));
        start = parent;
        current = parent;
    }
    col
}
