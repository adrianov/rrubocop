use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver};
use crate::parse::source::SourceFile;

use super::common::{binary_op, literal_or_const, same_target, sides, unwrap};

fn match_side<'a>(
    source: &'a SourceFile,
    l: Node<'a>,
    r: Node<'a>,
    target: Node<'a>,
) -> Option<Node<'a>> {
    let l = unwrap(l);
    let r = unwrap(r);
    if same_target(source, l, target) && r.kind() == "regex" {
        return Some(r);
    }
    if same_target(source, r, target) && l.kind() == "regex" {
        return Some(l);
    }
    None
}

fn equality_side<'a>(
    source: &'a SourceFile,
    l: Node<'a>,
    r: Node<'a>,
    target: Node<'a>,
) -> Option<Node<'a>> {
    let l = unwrap(l);
    let r = unwrap(r);
    if same_target(source, l, target) && literal_or_const(source, r) {
        return Some(r);
    }
    if same_target(source, r, target) && literal_or_const(source, l) {
        return Some(l);
    }
    None
}

fn collect_binary_condition(source: &SourceFile, n: Node<'_>, target: Node<'_>) -> bool {
    let op = binary_op(source, n).unwrap_or(b"");
    let (l, r) = sides(n).unwrap();
    match op {
        b"=~" => match_side(source, l, r, target).is_some(),
        b"==" | b"eql?" | b"equal?" | b"===" => equality_side(source, l, r, target).is_some(),
        _ => false,
    }
}

fn collect_call_condition(source: &SourceFile, n: Node<'_>, target: Node<'_>) -> bool {
    let meth = call_method_name(source, n).unwrap_or(b"");
    let recv = call_receiver(n);
    let arg = crate::cop::shared::argument_nodes(n).first().copied();
    match meth {
        b"=~" | b"match" | b"match?" => match (recv, arg) {
            (Some(r), Some(a)) => match_side(source, r, a, target).is_some(),
            _ => false,
        },
        b"is_a?" => recv.is_some_and(|r| same_target(source, r, target)),
        _ => false,
    }
}

pub(super) fn collect_condition(source: &SourceFile, node: Node<'_>, target: Node<'_>) -> bool {
    let n = unwrap(node);
    if n.kind() == "binary" && binary_op(source, n) == Some(b"||") {
        let (l, r) = sides(n).unwrap();
        return collect_condition(source, l, target) && collect_condition(source, r, target);
    }
    if binary_op(source, n).is_some() {
        return collect_binary_condition(source, n, target);
    }
    if n.kind() == "call" {
        return collect_call_condition(source, n, target);
    }
    false
}
