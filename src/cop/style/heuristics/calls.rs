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
    // Compact style is `scope_resolution`. Skip only single-segment cbase (`::Foo`),
    // matching RuboCop `namespace&.cbase_type?` — not `Admin::Members` / `::A::B`.
    if name.kind() != "scope_resolution" || single_cbase(name) {
        return false;
    }
    config.get_str("EnforcedStyle", "nested") == "nested"
}

fn single_cbase(name: Node<'_>) -> bool {
    name.child_by_field_name("scope").is_none()
}
