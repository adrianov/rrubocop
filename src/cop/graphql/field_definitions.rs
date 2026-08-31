//! GraphQL/FieldDefinitions — group fields or place resolvers after definitions.

use tree_sitter::Node;

use super::helpers::{
    class_body_stmts, field_has_explicit_resolver, field_name, is_field_call, module_body_stmts,
    nested_class, resolver_method_name, DEPT_INCLUDE,
};
use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FieldDefinitions;

impl Cop for FieldDefinitions {
    fn name(&self) -> &'static str {
        "GraphQL/FieldDefinitions"
    }

    fn default_include(&self) -> &'static [&'static str] {
        DEPT_INCLUDE
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class", "module"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.kind() == "class" && nested_class(node) {
            return;
        }
        let body = if node.kind() == "module" {
            module_body_stmts(node)
        } else {
            class_body_stmts(node)
        };
        if config.get_str("EnforcedStyle", "group_definitions") == "define_resolver_after_definition"
        {
            check_resolver_after(self, source, &body, diagnostics);
        } else {
            check_grouped(self, source, &body, diagnostics);
        }
    }
}

fn check_grouped(
    cop: &FieldDefinitions,
    source: &SourceFile,
    body: &[Node<'_>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen_field = false;
    let mut seen_other_after = false;
    for &n in body {
        if n.kind() == "comment" {
            continue;
        }
        if is_field_call(source, n) {
            if seen_other_after {
                push_group_msg(cop, source, n, diagnostics);
            }
            seen_field = true;
        } else if seen_field {
            seen_other_after = true;
        }
    }
}

fn push_group_msg(
    cop: &FieldDefinitions,
    source: &SourceFile,
    n: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, col) = source.offset_to_line_col(n.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        "Group all field definitions together.".into(),
    ));
}

fn check_resolver_after(
    cop: &FieldDefinitions,
    source: &SourceFile,
    body: &[Node<'_>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (i, stmt) in body.iter().enumerate() {
        if let Some(msg) = resolver_placement_msg(source, body, i, *stmt) {
            let (line, col) = source.offset_to_line_col(stmt.start_byte());
            diagnostics.push(cop.diagnostic(source, line, col, msg.into()));
        }
    }
}

fn resolver_placement_msg(
    source: &SourceFile,
    body: &[Node<'_>],
    i: usize,
    stmt: Node<'_>,
) -> Option<&'static str> {
    if !last_field_needing_resolver(source, body, i, stmt) {
        return None;
    }
    let resolver = resolver_method_name(source, stmt);
    let method_idx = body.iter().position(|n| is_def_named(source, *n, &resolver))?;
    if method_idx == i + 1 {
        return None;
    }
    let name = field_name(source, stmt).unwrap_or_default();
    Some(if same_name_field_count(source, body, &name) == 1 {
        "Define resolver method after field definition."
    } else {
        "Define resolver method after last field definition sharing resolver method."
    })
}

fn last_field_needing_resolver(
    source: &SourceFile,
    body: &[Node<'_>],
    i: usize,
    stmt: Node<'_>,
) -> bool {
    if !is_field_call(source, stmt) || field_has_explicit_resolver(source, stmt) {
        return false;
    }
    let name = field_name(source, stmt).unwrap_or_default();
    last_same_name_idx(source, body, &name) == Some(i)
}

fn last_same_name_idx(source: &SourceFile, body: &[Node<'_>], name: &str) -> Option<usize> {
    body.iter()
        .enumerate()
        .filter(|(_, n)| {
            is_field_call(source, **n) && field_name(source, **n).as_deref() == Some(name)
        })
        .map(|(j, _)| j)
        .last()
}

fn same_name_field_count(source: &SourceFile, body: &[Node<'_>], name: &str) -> usize {
    body.iter()
        .filter(|n| is_field_call(source, **n) && field_name(source, **n).as_deref() == Some(name))
        .count()
}

fn is_def_named(source: &SourceFile, node: Node<'_>, name: &str) -> bool {
    node.kind() == "method"
        && node
            .child_by_field_name("name")
            .map(|n| node_text(source, n) == name)
            .unwrap_or(false)
}
