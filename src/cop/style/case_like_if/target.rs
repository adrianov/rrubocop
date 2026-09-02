use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver};
use crate::parse::source::SourceFile;

use super::common::{binary_op, literal_or_const, sides, unwrap};

fn match_target<'a>(_source: &'a SourceFile, lhs: Node<'a>, rhs: Node<'a>) -> Option<Node<'a>> {
    if lhs.kind() == "regex" {
        return (rhs.kind() == "identifier").then_some(rhs);
    }
    if rhs.kind() == "regex" {
        return (lhs.kind() == "identifier").then_some(lhs);
    }
    None
}

fn equality_target<'a>(source: &'a SourceFile, l: Node<'a>, r: Node<'a>) -> Option<Node<'a>> {
    let l = unwrap(l);
    let r = unwrap(r);
    if literal_or_const(source, l) && r.kind() == "identifier" {
        return Some(r);
    }
    if literal_or_const(source, r) && l.kind() == "identifier" {
        return Some(l);
    }
    None
}

fn find_target_binary<'a>(source: &'a SourceFile, n: Node<'a>) -> Option<Node<'a>> {
    let op = binary_op(source, n)?;
    let (l, r) = sides(n)?;
    match op {
        b"=~" => match_target(source, l, r),
        b"==" | b"eql?" | b"equal?" | b"===" => equality_target(source, l, r),
        _ => None,
    }
}

fn find_target_call<'a>(source: &'a SourceFile, n: Node<'a>) -> Option<Node<'a>> {
    let meth = call_method_name(source, n)?;
    let recv = call_receiver(n)?;
    let args = crate::cop::shared::argument_nodes(n);
    let arg = args.first().copied()?;
    match meth {
        b"=~" | b"match" | b"match?" => match_target(source, recv, arg),
        b"is_a?" if recv.kind() == "identifier" => Some(arg),
        b"include?" | b"cover?" if recv.kind() == "range" => Some(recv),
        _ => None,
    }
}

pub(super) fn find_target<'a>(source: &'a SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    let n = unwrap(node);
    if n.kind() == "call" {
        return find_target_call(source, n);
    }
    if n.kind() != "binary" {
        return None;
    }
    if binary_op(source, n) == Some(b"||") {
        return find_target(source, sides(n)?.0);
    }
    find_target_binary(source, n)
}
