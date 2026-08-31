//! GraphQL/ObjectDescription — types need a description.

use tree_sitter::Node;

use super::helpers::{
    class_body_stmts, description_method_in, module_body_stmts, DEPT_INCLUDE,
};
use crate::cop::shared::{argument_nodes, call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ObjectDescription;

impl Cop for ObjectDescription {
    fn name(&self) -> &'static str {
        "GraphQL/ObjectDescription"
    }

    fn default_include(&self) -> &'static [&'static str] {
        DEPT_INCLUDE
    }

    fn default_exclude(&self) -> &'static [&'static str] {
        &[
            "spec/**/*",
            "test/**/*",
            "**/*_schema.rb",
            "**/base_*.rb",
            "**/graphql/query_context.rb",
        ]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class", "module"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(stmts) = body_to_check(source, node) else {
            return;
        };
        if description_method_in(source, &stmts) {
            return;
        }
        let name_node = node.child_by_field_name("name").unwrap_or(node);
        let (line, col) = source.offset_to_line_col(name_node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Missing type description".into(),
        ));
    }
}

fn body_to_check<'a>(source: &SourceFile, node: Node<'a>) -> Option<Vec<Node<'a>>> {
    if node.kind() == "module" {
        let kids = module_body_stmts(node);
        kids.iter().any(|n| is_include_call(source, *n)).then_some(kids)
    } else {
        Some(class_body_stmts(node))
    }
}

fn is_include_call(source: &SourceFile, node: Node<'_>) -> bool {
    call_method_name(source, node) == Some(b"include") && !argument_nodes(node).is_empty()
}
