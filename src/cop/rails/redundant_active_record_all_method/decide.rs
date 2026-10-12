//! Whether `all` is a redundant Active Record receiver.

use tree_sitter::Node;

use crate::cop::CopConfig;
use crate::cop::shared::{call_method_name, call_receiver, method_node, node_bytes, node_text};
use crate::parse::source::SourceFile;

/// `ActiveRecord::Querying::QUERYING_METHODS` (Active Record 7.1). Sorted for search.
pub(super) const QUERYING: &[&[u8]] = &[
    b"and",
    b"annotate",
    b"any?",
    b"async_average",
    b"async_count",
    b"async_ids",
    b"async_maximum",
    b"async_minimum",
    b"async_pick",
    b"async_pluck",
    b"async_sum",
    b"average",
    b"calculate",
    b"count",
    b"create_or_find_by",
    b"create_or_find_by!",
    b"create_with",
    b"delete_all",
    b"delete_by",
    b"destroy_all",
    b"destroy_by",
    b"distinct",
    b"eager_load",
    b"except",
    b"excluding",
    b"exists?",
    b"extending",
    b"extract_associated",
    b"fifth",
    b"fifth!",
    b"find",
    b"find_by",
    b"find_by!",
    b"find_each",
    b"find_in_batches",
    b"find_or_create_by",
    b"find_or_create_by!",
    b"find_or_initialize_by",
    b"find_sole_by",
    b"first",
    b"first!",
    b"first_or_create",
    b"first_or_create!",
    b"first_or_initialize",
    b"forty_two",
    b"forty_two!",
    b"fourth",
    b"fourth!",
    b"from",
    b"group",
    b"having",
    b"ids",
    b"in_batches",
    b"in_order_of",
    b"includes",
    b"invert_where",
    b"joins",
    b"last",
    b"last!",
    b"left_joins",
    b"left_outer_joins",
    b"limit",
    b"lock",
    b"many?",
    b"maximum",
    b"merge",
    b"minimum",
    b"none",
    b"none?",
    b"offset",
    b"one?",
    b"only",
    b"optimizer_hints",
    b"or",
    b"order",
    b"pick",
    b"pluck",
    b"preload",
    b"readonly",
    b"references",
    b"regroup",
    b"reorder",
    b"reselect",
    b"rewhere",
    b"second",
    b"second!",
    b"second_to_last",
    b"second_to_last!",
    b"select",
    b"sole",
    b"strict_loading",
    b"sum",
    b"take",
    b"take!",
    b"third",
    b"third!",
    b"third_to_last",
    b"third_to_last!",
    b"touch_all",
    b"unscope",
    b"update_all",
    b"where",
    b"with",
    b"without",
];

pub(super) struct All<'a> {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) receiver: Option<Node<'a>>,
}

pub(super) struct Hit<'a> {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) op: Node<'a>,
}

fn is_call(node: Node<'_>) -> bool {
    matches!(node.kind(), "call" | "command" | "command_call")
}

fn is_const(node: Node<'_>) -> bool {
    matches!(node.kind(), "constant" | "scope_resolution")
}

pub(super) fn is_querying(name: &[u8]) -> bool {
    QUERYING.binary_search(&name).is_ok()
}

fn zero_args(node: Node<'_>) -> bool {
    node.child_by_field_name("arguments")
        .is_none_or(|args| args.named_child_count() == 0)
}

/// Dot (or `::`) that joins `all` to the query method. A `do` block call
/// inlines the chain and leaves the dot unnamed.
fn query_op<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    let recv_end = call_receiver(node)?.end_byte();
    let dot = |op: Node<'_>| {
        op.start_byte() >= recv_end && matches!(node_bytes(source, op), b"." | b"::")
    };
    if let Some(op) = node.child_by_field_name("operator")
        && dot(op)
    {
        return Some(op);
    }
    let mut cur = node.walk();
    node.children(&mut cur).find(|child| dot(*child))
}

/// `User.all` / `User.all()` — not `all(:x)` and not a bare identifier.
pub(super) fn call_all<'a>(source: &SourceFile, recv: Node<'a>) -> Option<All<'a>> {
    if !is_call(recv) || call_method_name(source, recv) != Some(b"all") || !zero_args(recv) {
        return None;
    }
    if recv.child_by_field_name("block").is_some() {
        return None;
    }
    let meth = method_node(recv)?;
    Some(All {
        start: meth.start_byte(),
        end: recv.end_byte(),
        receiver: call_receiver(recv),
    })
}

fn block_pass(node: Node<'_>) -> bool {
    node.child_by_field_name("arguments")
        .and_then(|args| args.named_child(0))
        .is_some_and(|arg| arg.kind() == "block_argument")
}

fn enumerable_block(source: &SourceFile, query: Node<'_>) -> bool {
    let Some(name) = call_method_name(source, query) else {
        return false;
    };
    if !matches!(
        name,
        b"any?" | b"count" | b"find" | b"none?" | b"one?" | b"select" | b"sum"
    ) {
        return false;
    }
    query.child_by_field_name("block").is_some() || block_pass(query)
}

fn sensitive(source: &SourceFile, query: Node<'_>, receiver: Option<Node<'_>>) -> bool {
    let Some(name) = call_method_name(source, query) else {
        return false;
    };
    matches!(name, b"delete_all" | b"destroy_all") && !receiver.is_some_and(is_const)
}

fn receiver_name(source: &SourceFile, recv: Node<'_>) -> String {
    if let Some(inner) = call_receiver(recv)
        && !is_const(inner)
    {
        return receiver_name(source, inner);
    }
    if !is_call(recv) {
        return node_text(source, recv);
    }
    let method = call_method_name(source, recv).unwrap_or_default();
    let method = String::from_utf8_lossy(method);
    match call_receiver(recv) {
        Some(inner) => format!("{}.{}", receiver_name(source, inner), method),
        None => method.into_owned(),
    }
}

fn allowed(source: &SourceFile, recv: Node<'_>, config: &CopConfig) -> bool {
    let name = receiver_name(source, recv);
    let Some(seq) = config
        .options
        .get("AllowedReceivers")
        .and_then(|v| v.as_sequence())
    else {
        return name == "ActionMailer::Preview" || name == "ActiveSupport::TimeZone";
    };
    seq.iter().any(|v| v.as_str() == Some(name.as_str()))
}

fn ar_superclass(source: &SourceFile, class_node: Node<'_>) -> bool {
    let Some(sc) = class_node.child_by_field_name("superclass") else {
        return false;
    };
    let mut cur = sc.walk();
    sc.named_children(&mut cur).next().is_some_and(|expr| {
        let name = node_text(source, expr);
        let name = name.trim().trim_start_matches("::");
        name == "ApplicationRecord" || name == "ActiveRecord::Base"
    })
}

fn in_active_record(source: &SourceFile, node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(n) = p {
        if n.kind() == "class" && ar_superclass(source, n) {
            return true;
        }
        p = n.parent();
    }
    false
}

pub(super) fn keep<'a>(
    source: &SourceFile,
    query: Node<'a>,
    config: &CopConfig,
    all: All<'a>,
) -> Option<Hit<'a>> {
    let op = query_op(source, query)?;
    if enumerable_block(source, query) || sensitive(source, query, all.receiver) {
        return None;
    }
    if let Some(recv) = all.receiver {
        if allowed(source, recv, config) {
            return None;
        }
    } else if !in_active_record(source, query) {
        return None;
    }
    Some(Hit {
        start: all.start,
        end: all.end,
        op,
    })
}
