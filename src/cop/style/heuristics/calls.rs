//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_class_and_module_children(_source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if !matches!(node.kind(), "class" | "module") {
        return false;
    }
    if node
        .parent()
        .is_some_and(|p| matches!(p.kind(), "class" | "module"))
    {
        return false;
    }
    let Some(name) = node.child_by_field_name("name") else {
        return false;
    };
    if name.kind() != "scope_resolution" || cbase_scope(name) {
        return false;
    }
    config.get_str("EnforcedStyle", "nested") == "nested"
}

fn cbase_scope(name: Node<'_>) -> bool {
    let mut cur = name.walk();
    name.named_children(&mut cur)
        .next()
        .is_some_and(|n| n.kind() == "constant")
}
