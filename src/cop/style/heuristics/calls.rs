//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_alias(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    let style = config.get_str("EnforcedStyle", "prefer_alias");
    match node.kind() {
        "alias" => style == "prefer_alias_method",
        "call" => {
            crate::cop::shared::call_method_name(source, node) == Some(b"alias_method")
                && style == "prefer_alias"
                && crate::cop::shared::call_receiver(node).is_none()
        }
        _ => false,
    }
}

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

pub fn matches_class_methods(_source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    // class << self with only method defs — flag def self.x instead preference opposite
    // Prefer `def self.x` over singleton class for single method — detect singleton_class
    node.kind() == "singleton_class"
}

pub fn matches_date_time(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{call_receiver, is_const_named, node_bytes};
    match node.kind() {
        "constant" => node_bytes(source, node) == b"DateTime",
        "call" => call_receiver(node).is_some_and(|r| is_const_named(source, r, b"DateTime")),
        _ => false,
    }
}

pub fn matches_global_vars(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::node_bytes;
    if node.kind() != "assignment" && node.kind() != "operator_assignment" { return false; }
    let Some(left) = node.child(0) else { return false; };
    left.kind() == "global_variable" && {
        let b = node_bytes(source, left);
        // allow English / std streams
        !matches!(b, b"$stdin" | b"$stdout" | b"$stderr" | b"$LOADED_FEATURES" | b"$LOAD_PATH" | b"$PROGRAM_NAME" | b"$FILENAME" | b"$SAFE" | b"$DEBUG" | b"$VERBOSE")
    }
}

pub fn matches_module_function(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{call_method_name, call_receiver};
    call_method_name(source, node) == Some(b"module_function") && call_receiver(node).is_none()
}

pub fn matches_nested_file_dirname(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, is_const_named};
    if call_method_name(source, node) != Some(b"dirname") { return false; }
    if !call_receiver(node).is_some_and(|r| is_const_named(source, r, b"File")) { return false; }
    let args = argument_nodes(node);
    if args.is_empty() { return false; }
    let a0 = args[0];
    a0.kind() == "call" && call_method_name(source, a0) == Some(b"dirname")
        && call_receiver(a0).is_some_and(|r| is_const_named(source, r, b"File"))
}

pub fn matches_open_struct_use(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{call_method_name, call_receiver, is_const_named, node_bytes};
    match node.kind() {
        "constant" => node_bytes(source, node) == b"OpenStruct",
        "call" => call_method_name(source, node) == Some(b"new")
            && call_receiver(node).is_some_and(|r| is_const_named(source, r, b"OpenStruct")),
        _ => false,
    }
}

pub fn matches_preferred_hash_methods(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    matches!(crate::cop::shared::call_method_name(source, node), Some(b"has_key?" | b"has_value?"))
}

pub fn matches_proc(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{call_method_name, call_receiver, is_const_named};
    call_method_name(source, node) == Some(b"new")
        && call_receiver(node).is_some_and(|r| is_const_named(source, r, b"Proc"))
}

pub fn matches_raise_args(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    use crate::cop::shared::{argument_nodes, call_method_name, call_receiver};
    if !matches!(call_method_name(source, node), Some(b"raise" | b"fail")) { return false; }
    if call_receiver(node).is_some() { return false; }
    let style = config.get_str("EnforcedStyle", "exploded");
    let args = argument_nodes(node);
    match style {
        "exploded" => args.len() == 1 && is_exception_new(source, args[0]),
        "compact" => args.len() >= 2,
        _ => false,
    }
}

fn is_exception_new(source: &SourceFile, arg: Node<'_>) -> bool {
    use crate::cop::shared::call_method_name;
    arg.kind() == "call" && call_method_name(source, arg) == Some(b"new")
}
pub fn matches_strip(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    use crate::cop::shared::{call_method_name, call_receiver};
    let Some(m) = call_method_name(source, node) else { return false; };
    if !matches!(m, b"lstrip" | b"rstrip") { return false; }
    let Some(recv) = call_receiver(node) else { return false; };
    if recv.kind() != "call" { return false; }
    let Some(rm) = call_method_name(source, recv) else { return false; };
    (m == b"lstrip" && rm == b"rstrip") || (m == b"rstrip" && rm == b"lstrip")
}

