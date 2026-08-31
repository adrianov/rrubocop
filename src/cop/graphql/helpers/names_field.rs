//! Field / argument name helpers for GraphQL Ruby DSL cops.

use tree_sitter::Node;

use crate::parse::source::SourceFile;

use super::args::first_sym_arg;
use super::kwargs::{has_kwarg, kwarg_sym_value};

pub fn field_name(source: &SourceFile, node: Node<'_>) -> Option<String> {
    first_sym_arg(source, node)
}

pub fn argument_name(source: &SourceFile, node: Node<'_>) -> Option<String> {
    first_sym_arg(source, node)
}

pub fn resolver_method_name(source: &SourceFile, field: Node<'_>) -> String {
    kwarg_sym_value(source, field, "resolver_method")
        .or_else(|| field_name(source, field))
        .unwrap_or_default()
}

pub fn field_has_explicit_resolver(source: &SourceFile, field: Node<'_>) -> bool {
    has_kwarg(source, field, "resolver")
        || has_kwarg(source, field, "method")
        || has_kwarg(source, field, "hash_key")
}

pub fn inferred_arg_name(name: &str) -> String {
    if let Some(base) = name.strip_suffix("_ids") {
        let mut s = base.to_string();
        if !s.ends_with('s') {
            s.push('s');
        }
        return s;
    }
    name.strip_suffix("_id")
        .map(|b| b.to_string())
        .unwrap_or_else(|| name.to_string())
}

pub fn effective_arg_name(source: &SourceFile, arg: Node<'_>) -> Option<String> {
    if let Some(as_name) = kwarg_sym_value(source, arg, "as") {
        return Some(as_name);
    }
    let name = argument_name(source, arg)?;
    if has_kwarg(source, arg, "loads") {
        Some(inferred_arg_name(&name))
    } else {
        Some(name)
    }
}
