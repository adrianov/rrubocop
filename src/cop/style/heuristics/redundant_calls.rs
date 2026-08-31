//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_redundant_array_constructor(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, is_const_named};
    // Array() or Array[] or Array.new([])
    if let Some(recv) = call_receiver(node) {
        if is_const_named(source, recv, b"Array") && call_method_name(source, node) == Some(b"new") {
            let args = argument_nodes(node);
            return args.len() == 1 && args[0].kind() == "array";
        }
        return false;
    }
    // Array(...) as kernel-ish — call with method Array
    call_method_name(source, node) == Some(b"Array")
}

pub fn matches_redundant_current_directory_in_path(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_bytes};
    if !matches!(call_method_name(source, node), Some(b"require" | b"require_relative" | b"load" | b"autoload")) { return false; }
    if call_receiver(node).is_some() { return false; }
    let args = argument_nodes(node);
    if args.is_empty() { return false; }
    let b = node_bytes(source, args[0]);
    // './foo' or "./foo"
    (b.starts_with(b"'./") || b.starts_with(b"\"./")) && call_method_name(source, node) == Some(b"require")
}

pub fn matches_redundant_each(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{call_method_name, call_receiver};
    if call_method_name(source, node) != Some(b"each") { return false; }
    let Some(recv) = call_receiver(node) else { return false; };
    recv.kind() == "call" && matches!(call_method_name(source, recv), Some(b"each" | b"each_with_index" | b"reverse_each"))
}

pub fn matches_redundant_exception(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, is_const_named};
    if !matches!(call_method_name(source, node), Some(b"raise" | b"fail")) { return false; }
    if call_receiver(node).is_some() { return false; }
    let args = argument_nodes(node);
    if args.len() < 2 { return false; }
    is_const_named(source, args[0], b"RuntimeError")
}

pub fn matches_redundant_filter_chain(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{call_method_name, call_receiver};
    // select{}.first / find_all{}.last etc
    if !matches!(call_method_name(source, node), Some(b"first" | b"last" | b"take")) { return false; }
    let Some(recv) = call_receiver(node) else { return false; };
    recv.kind() == "call" && matches!(call_method_name(source, recv), Some(b"select" | b"find_all" | b"filter" | b"reject"))
}

pub fn matches_redundant_regexp_constructor(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, is_const_named};
    if call_method_name(source, node) != Some(b"new") && call_method_name(source, node) != Some(b"compile") { return false; }
    let Some(recv) = call_receiver(node) else { return false; };
    if !is_const_named(source, recv, b"Regexp") { return false; }
    let args = argument_nodes(node);
    !args.is_empty() && matches!(args[0].kind(), "string" | "regex")
}

pub fn matches_redundant_sort_by(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::call_method_name;
    if call_method_name(source, node) != Some(b"sort_by") { return false; }
    let mut cur = node.walk();
    let Some(block) = node.children(&mut cur).find(|ch| matches!(ch.kind(), "block" | "do_block")) else {
        return false;
    };
    identity_block(source, block)
}

fn identity_block(source: &SourceFile, block: Node<'_>) -> bool {
    let (Some(p), Some(b)) = (block_params(block), block_body(block)) else { return false; };
    same_single_ident(source, p, b)
}

fn block_params(block: Node<'_>) -> Option<Node<'_>> {
    let mut c2 = block.walk();
    block.children(&mut c2).find(|ch| ch.kind() == "block_parameters")
}

fn same_single_ident(source: &SourceFile, params: Node<'_>, body: Node<'_>) -> bool {
    let mut cp = params.walk();
    let pn: Vec<_> = params.named_children(&mut cp).collect();
    let mut cb = body.walk();
    let bn: Vec<_> = body.named_children(&mut cb).collect();
    pn.len() == 1 && bn.len() == 1
        && crate::cop::shared::node_bytes(source, pn[0]) == crate::cop::shared::node_bytes(source, bn[0])
}

fn block_body(block: Node<'_>) -> Option<Node<'_>> {
    block.child_by_field_name("body").or_else(|| {
        let mut c3 = block.walk();
        block.children(&mut c3).find(|ch| matches!(ch.kind(), "block_body" | "body_statement"))
    })
}
