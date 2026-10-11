//! One walk for block passes and local uses shared by the forwarding cops.
//!
//! `excluded_parents` skips identifier parents that are not a real use.
//! `scope_stops` ends the ancestor search (the enclosing method).
//! `include_lambda` counts a `lambda` node as a block. `skip_scopes` are
//! nested def/class/module nodes the walk does not enter.

use tree_sitter::Node;

use super::node_bytes;
use crate::parse::source::SourceFile;

pub const EXCLUDED_BLOCK_PARENTS: &[&str] = &["block_argument", "block_parameter"];
pub const METHOD_STOPS: &[&str] = &["method", "singleton_method"];
pub const NESTED_SCOPES: &[&str] = &["method", "singleton_method", "class", "module"];

pub struct ScanOpts<'a> {
    pub skip_scopes: &'a [&'a str],
    pub excluded_parents: &'a [&'a str],
    pub scope_stops: &'a [&'a str],
    pub include_lambda: bool,
    /// Any block pass inside a block sets `blocked`, not only a selected one.
    pub any_inside_block: bool,
    /// When set, a pass inside a block clears the result (`blocked`).
    pub guard_inside_block: bool,
}

pub struct Passes<'a> {
    pub nodes: Vec<Node<'a>>,
    pub blocked: bool,
    pub name_used: bool,
}

pub fn scan<'a>(
    source: &SourceFile,
    body: Node<'a>,
    use_name: &[u8],
    pass_name: Option<&[u8]>,
    opts: &ScanOpts<'_>,
) -> Passes<'a> {
    let mut out = Passes {
        nodes: Vec::new(),
        blocked: false,
        name_used: false,
    };
    let mut stack = vec![body];
    while let Some(node) = stack.pop() {
        if node.id() != body.id() && opts.skip_scopes.contains(&node.kind()) {
            continue;
        }
        if node.kind() == "identifier" {
            out.name_used |= ident_used(source, node, use_name, opts.excluded_parents);
        } else if node.kind() == "block_argument"
            && take_pass(source, node, pass_name, opts, &mut out)
        {
            out.nodes.clear();
            out.blocked = true;
            return out;
        }
        push_children(&mut stack, node);
    }
    out
}

fn ident_used(source: &SourceFile, node: Node<'_>, name: &[u8], excluded: &[&str]) -> bool {
    node_bytes(source, node) == name
        && !node
            .parent()
            .is_some_and(|parent| excluded.contains(&parent.kind()))
}

fn take_pass<'a>(
    source: &SourceFile,
    node: Node<'a>,
    pass_name: Option<&[u8]>,
    opts: &ScanOpts<'_>,
    out: &mut Passes<'a>,
) -> bool {
    let selected = selected_pass(source, node, pass_name);
    if opts.guard_inside_block
        && (opts.any_inside_block || selected)
        && in_block(node, opts.include_lambda, opts.scope_stops)
    {
        return true;
    }
    if selected {
        out.nodes.push(node);
    }
    false
}

fn selected_pass(source: &SourceFile, node: Node<'_>, name: Option<&[u8]>) -> bool {
    match name {
        None => node.named_child_count() == 0,
        Some(name) => pass_ident(source, node) == Some(name),
    }
}

fn pass_ident<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .find(|child| child.kind() == "identifier")
        .map(|child| node_bytes(source, child))
}

fn in_block(node: Node<'_>, include_lambda: bool, scope_stops: &[&str]) -> bool {
    let mut parent = node.parent();
    while let Some(current) = parent {
        if block_node(current, include_lambda) {
            return true;
        }
        if scope_stops.contains(&current.kind()) {
            return false;
        }
        parent = current.parent();
    }
    false
}

fn block_node(node: Node<'_>, include_lambda: bool) -> bool {
    matches!(node.kind(), "block" | "do_block") || (include_lambda && node.kind() == "lambda")
}

fn push_children<'a>(stack: &mut Vec<Node<'a>>, node: Node<'a>) {
    let mut cur = node.walk();
    stack.extend(node.children(&mut cur));
}
