//! GraphQL/MultipleFieldDefinitions — group duplicate-named field definitions.

use tree_sitter::Node;

use super::helpers::{
    class_body_stmts, enclosing_class, field_name, is_field_call, nested_class, CALL_KINDS,
    DEPT_INCLUDE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultipleFieldDefinitions;

impl Cop for MultipleFieldDefinitions {
    fn name(&self) -> &'static str {
        "GraphQL/MultipleFieldDefinitions"
    }

    fn default_include(&self) -> &'static [&'static str] {
        DEPT_INCLUDE
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        CALL_KINDS
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_field_call(source, node) {
            return;
        }
        let Some(class) = enclosing_class(node) else {
            return;
        };
        if nested_class(class) {
            return;
        }
        let Some(name) = field_name(source, node) else {
            return;
        };
        if !ungrouped_last_def(source, class, node, &name) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Group multiple field definitions together.".into(),
        ));
    }
}

fn ungrouped_last_def(
    source: &SourceFile,
    class: Node<'_>,
    node: Node<'_>,
    name: &str,
) -> bool {
    let defs = same_name_defs(source, class, name);
    is_last_ungrouped(&defs, node)
}

fn same_name_defs<'a>(
    source: &SourceFile,
    class: Node<'a>,
    name: &str,
) -> Vec<(usize, Node<'a>)> {
    class_body_stmts(class)
        .into_iter()
        .enumerate()
        .filter(|(_, n)| {
            is_field_call(source, *n) && field_name(source, *n).as_deref() == Some(name)
        })
        .map(|(i, n)| (i, n))
        .collect()
}

fn is_last_ungrouped(defs: &[(usize, Node<'_>)], node: Node<'_>) -> bool {
    defs.len() > 1
        && defs.last().map(|(_, n)| n.id()) == Some(node.id())
        && defs.windows(2).any(|w| w[1].0 - w[0].0 > 1)
}
