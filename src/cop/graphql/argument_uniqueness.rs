//! GraphQL/ArgumentUniqueness — duplicate arguments in the same scope.

use std::collections::{HashMap, HashSet};

use tree_sitter::Node;

use super::helpers::{
    argument_name, enclosing_class, field_name, is_argument_call, is_field_call, nested_class,
    DEPT_INCLUDE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ArgumentUniqueness;

impl Cop for ArgumentUniqueness {
    fn name(&self) -> &'static str {
        "GraphQL/ArgumentUniqueness"
    }

    fn default_include(&self) -> &'static [&'static str] {
        DEPT_INCLUDE
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if nested_class(node) {
            return;
        }
        let mut by_field: HashMap<String, HashSet<String>> = HashMap::new();
        walk(self, source, node, node, &mut by_field, diagnostics);
    }
}

fn walk(
    cop: &ArgumentUniqueness,
    source: &SourceFile,
    node: Node<'_>,
    class_node: Node<'_>,
    by_field: &mut HashMap<String, HashSet<String>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "class" && node.id() != class_node.id() {
        return;
    }
    if is_argument_call(source, node) {
        check_dup_arg(cop, source, node, class_node, by_field, diagnostics);
    }
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        walk(cop, source, child, class_node, by_field, diagnostics);
    }
}

fn check_dup_arg(
    cop: &ArgumentUniqueness,
    source: &SourceFile,
    node: Node<'_>,
    class_node: Node<'_>,
    by_field: &mut HashMap<String, HashSet<String>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(name) = arg_in_class(source, node, class_node) else {
        return;
    };
    let field_key = parent_field_key(source, node).unwrap_or_else(|| "root".into());
    if by_field.entry(field_key.clone()).or_default().insert(name.clone()) {
        return;
    }
    push_dup(cop, source, node, &name, &field_key, diagnostics);
}

fn arg_in_class(source: &SourceFile, node: Node<'_>, class_node: Node<'_>) -> Option<String> {
    let enc = enclosing_class(node)?;
    (enc.id() == class_node.id())
        .then(|| argument_name(source, node))
        .flatten()
}

fn push_dup(
    cop: &ArgumentUniqueness,
    source: &SourceFile,
    node: Node<'_>,
    name: &str,
    field_key: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let field_msg = if field_key == "root" {
        String::new()
    } else {
        format!(" in field `{field_key}`")
    };
    let (line, col) = source.offset_to_line_col(node.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!(
            "Argument names should only be defined once per block. Argument `{name}` is duplicated{field_msg}."
        ),
    ));
}

fn parent_field_key(source: &SourceFile, arg: Node<'_>) -> Option<String> {
    let mut p = arg.parent();
    while let Some(n) = p {
        if n.kind() == "class" {
            return None;
        }
        if is_field_call(source, n) {
            return field_name(source, n);
        }
        if matches!(n.kind(), "do_block" | "block") {
            if let Some(call) = n.parent() {
                if is_field_call(source, call) {
                    return field_name(source, call);
                }
            }
        }
        p = n.parent();
    }
    None
}
