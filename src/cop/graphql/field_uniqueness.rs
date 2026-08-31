//! GraphQL/FieldUniqueness — duplicate fields in a type.

use std::collections::HashSet;

use tree_sitter::Node;

use super::helpers::{
    class_body_stmts, field_name, is_field_call, kwarg_false, nested_class, DEPT_INCLUDE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FieldUniqueness;

impl Cop for FieldUniqueness {
    fn name(&self) -> &'static str {
        "GraphQL/FieldUniqueness"
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
        let mut seen = HashSet::new();
        for stmt in class_body_stmts(node) {
            report_dup(self, source, stmt, &mut seen, diagnostics);
        }
    }
}

fn report_dup(
    cop: &FieldUniqueness,
    source: &SourceFile,
    stmt: Node<'_>,
    seen: &mut HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !is_field_call(source, stmt) {
        return;
    }
    let Some(name) = field_name(source, stmt) else {
        return;
    };
    let key = if kwarg_false(source, stmt, "camelize") {
        format!("{name}:non-camelized")
    } else {
        name.clone()
    };
    if !seen.insert(key) {
        let (line, col) = source.offset_to_line_col(stmt.start_byte());
        diagnostics.push(cop.diagnostic(
            source,
            line,
            col,
            format!(
                "Field names should only be defined once per type. Field `{name}` is duplicated."
            ),
        ));
    }
}
