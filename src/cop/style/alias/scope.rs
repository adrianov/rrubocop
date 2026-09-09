//! Style/Alias — scope classification for `alias` / `alias_method`.

use tree_sitter::Node;

use crate::cop::shared::call_method_name;
use crate::parse::source::SourceFile;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Scope {
    Lexical,
    Dynamic,
    InstanceEval,
}

fn is_instance_eval_block(source: &SourceFile, block: Node<'_>) -> bool {
    block
        .parent()
        .and_then(|call| call_method_name(source, call))
        .is_some_and(|n| n == b"instance_eval")
}

pub(super) fn lexical_scope_label(node: Node<'_>) -> &'static str {
    let mut p = node.parent();
    while let Some(cur) = p {
        match cur.kind() {
            "class" => return "in a class body",
            "module" => return "in a module body",
            _ => {}
        }
        p = cur.parent();
    }
    "at the top level"
}

pub(super) fn scope_of(source: &SourceFile, mut node: Node<'_>) -> Scope {
    while let Some(parent) = node.parent() {
        match parent.kind() {
            "class" | "module" => return Scope::Lexical,
            "method" | "singleton_method" => return Scope::Dynamic,
            "block" | "do_block" => {
                if is_instance_eval_block(source, parent) {
                    return Scope::InstanceEval;
                }
                return Scope::Dynamic;
            }
            _ => {}
        }
        node = parent;
    }
    Scope::Lexical
}

pub(super) fn inside_method_def(mut node: Node<'_>) -> bool {
    while let Some(parent) = node.parent() {
        if matches!(parent.kind(), "method" | "singleton_method") {
            return true;
        }
        node = parent;
    }
    false
}

pub(super) fn alias_method_value_used(node: Node<'_>) -> bool {
    node.parent()
        .is_some_and(|p| matches!(p.kind(), "argument_list" | "assignment" | "operator_assignment"))
}
