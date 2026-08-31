//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_class_and_module_children(_source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if !matches!(node.kind(), "class" | "module") { return false; }
    let Some(name) = node.child_by_field_name("name") else { return false; };
    let nested = name.kind() == "scope_resolution";
    let style = config.get_str("EnforcedStyle", "nested");
    match style {
        "nested" => nested, // compact form A::B should be nested class A; class B
        "compact" => !nested && false, // need nested class body — skip
        _ => false,
    }
}
