//! Inversion analysis for Style/InvertibleUnlessCondition.
//!
//! Port of RuboCop `Style/InvertibleUnlessCondition#invertible?`: an `unless`
//! condition is invertible when it is a method call whose method appears in
//! the cop's merged `InverseMethods` config (rubocop default.yml plus plugin
//! and project layers), a `!`/`not` negation, or an `and`/`or` chain whose
//! both sides are invertible. The rewrite itself lives in [`invert`].

mod invert;

use std::collections::HashMap;

use tree_sitter::Node;

use crate::cop::CopConfig;
use crate::cop::shared::{Edit, child_kind, field_text, node_text};
use crate::parse::source::SourceFile;

type Inverses<'a> = HashMap<&'a str, &'a str>;

/// For an invertible `unless`: its condition source, RuboCop's
/// `preferred_condition`, and the autocorrect edits (keyword included).
pub fn invert_unless(
    source: &SourceFile,
    node: Node<'_>,
    config: &CopConfig,
) -> Option<(String, String, Vec<Edit>)> {
    let condition = node.child_by_field_name("condition")?;
    let inverses = inverse_methods(config);
    if !is_invertible(condition, source, &inverses) {
        return None;
    }
    let keyword = child_kind(node, "unless")?;
    let mut edits = vec![(keyword.start_byte(), keyword.end_byte(), "if".to_string())];
    Some((
        node_text(source, condition),
        invert::invert(condition, source, &inverses, &mut edits),
        edits,
    ))
}

/// The cop's merged `InverseMethods` mapping without the YAML `:` sigil
/// (`:include?: :exclude?` → `include?` → `exclude?`).
fn inverse_methods(config: &CopConfig) -> Inverses<'_> {
    fn bare(v: &serde_yml::Value) -> Option<&str> {
        v.as_str().map(|s| s.trim_start_matches(':'))
    }
    config
        .options
        .get("InverseMethods")
        .and_then(serde_yml::Value::as_mapping)
        .map(|map| {
            map.iter()
                .filter_map(|(k, v)| Some((bare(k)?, bare(v)?)))
                .collect()
        })
        .unwrap_or_default()
}

fn is_invertible(node: Node<'_>, source: &SourceFile, inverses: &Inverses<'_>) -> bool {
    match node.kind() {
        // RuboCop unwraps parenthesized (`begin`) conditions to the first child.
        "parenthesized_statements" => node
            .named_child(0)
            .is_some_and(|first| is_invertible(first, source, inverses)),
        "call" => invertible_call(node, source, inverses),
        "unary" => matches!(operator(node, source), "!" | "not"),
        "binary" => invertible_binary(node, source, inverses),
        _ => false,
    }
}

fn invertible_binary(node: Node<'_>, source: &SourceFile, inverses: &Inverses<'_>) -> bool {
    match operator(node, source) {
        "&&" | "and" | "||" | "or" => {
            node.child_by_field_name("left")
                .is_some_and(|l| is_invertible(l, source, inverses))
                && node
                    .child_by_field_name("right")
                    .is_some_and(|r| is_invertible(r, source, inverses))
        }
        op => !inheritance_check(node, source) && inverses.contains_key(op),
    }
}

fn invertible_call(node: Node<'_>, source: &SourceFile, inverses: &Inverses<'_>) -> bool {
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
            name => inverses.contains_key(name),
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
    field_text(source, node, "operator")
}

fn text<'a>(node: Node<'_>, source: &'a SourceFile) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}
