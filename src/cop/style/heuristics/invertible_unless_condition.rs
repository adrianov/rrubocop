//! Inversion analysis for Style/InvertibleUnlessCondition.
//!
//! Port of RuboCop `Style/InvertibleUnlessCondition#invertible?`: an `unless`
//! condition is invertible when it is a method call whose method appears in
//! the cop's merged `InverseMethods` config (rubocop default.yml plus plugin
//! and project layers), a `!`/`not` negation, or an `and`/`or` chain whose
//! both sides are invertible.

use std::collections::HashSet;

use tree_sitter::Node;

use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_invertible_unless_condition(
    source: &SourceFile,
    node: Node<'_>,
    config: &CopConfig,
) -> bool {
    let Some(condition) = node.child_by_field_name("condition") else {
        return false;
    };
    is_invertible(condition, source, &inverse_methods(config))
}

/// Keys of the cop's merged `InverseMethods` mapping, normalized without the
/// YAML `:` sigil (`:include?` → `include?`).
fn inverse_methods(config: &CopConfig) -> HashSet<&str> {
    config
        .options
        .get("InverseMethods")
        .and_then(serde_yml::Value::as_mapping)
        .map(|map| {
            map.keys()
                .filter_map(serde_yml::Value::as_str)
                .map(|key| key.trim_start_matches(':'))
                .collect()
        })
        .unwrap_or_default()
}

fn is_invertible(node: Node<'_>, source: &SourceFile, inverses: &HashSet<&str>) -> bool {
    match node.kind() {
        // RuboCop unwraps parenthesized (`begin`) conditions to the first child.
        "begin" | "parenthesized_statements" => node
            .children(&mut node.walk())
            .find(|child| child.is_named())
            .is_some_and(|first| is_invertible(first, source, inverses)),
        "call" => invertible_call(node, source, inverses),
        "unary" => matches!(operator(node, source), "!" | "not"),
        "binary" => {
            let op = operator(node, source);
            if matches!(op, "&&" | "and" | "||" | "or") {
                let left = node.child_by_field_name("left");
                let right = node.child_by_field_name("right");
                left.is_some_and(|l| is_invertible(l, source, inverses))
                    && right.is_some_and(|r| is_invertible(r, source, inverses))
            } else {
                !inheritance_check(node, source) && inverses.contains(op)
            }
        }
        _ => false,
    }
}

fn invertible_call(node: Node<'_>, source: &SourceFile, inverses: &HashSet<&str>) -> bool {
    // `x.any? { }` is a :block node in RuboCop's AST — not a plain send.
    if node.child_by_field_name("block").is_some() {
        return false;
    }
    // Safe navigation is `csend` in RuboCop's AST — never invertible.
    if operator(node, source) == "&." {
        return false;
    }
    match node.child_by_field_name("method") {
        Some(method) => match text(method, source) {
            "!" | "not" => true,
            name => inverses.contains(name),
        },
        None => false,
    }
}

/// RuboCop `inheritance_check?`: `x < Const` with a non-screaming constant
/// name is a class-hierarchy check, not an invertible comparison.
fn inheritance_check(node: Node<'_>, source: &SourceFile) -> bool {
    if operator(node, source) != "<" {
        return false;
    }
    let Some(arg) = node.child_by_field_name("right") else {
        return false;
    };
    let Some(short) = short_constant_name(arg, source) else {
        return false;
    };
    short.to_uppercase() != short
}

/// Last segment of a constant reference: `Bar`, `::Bar`, or `A::B` → `B`.
fn short_constant_name<'a>(node: Node<'_>, source: &'a SourceFile) -> Option<&'a str> {
    match node.kind() {
        "constant" => Some(text(node, source)),
        "scope_resolution" => {
            let name = node.child_by_field_name("name")?;
            Some(text(name, source))
        }
        _ => None,
    }
}

fn operator<'a>(node: Node<'_>, source: &'a SourceFile) -> &'a str {
    node.child_by_field_name("operator")
        .map(|op| text(op, source))
        .unwrap_or("")
}

fn text<'a>(node: Node<'_>, source: &'a SourceFile) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}
