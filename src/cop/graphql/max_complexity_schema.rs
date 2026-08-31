//! GraphQL/MaxComplexitySchema — schema must set max_complexity.

use tree_sitter::Node;

use super::helpers::{collect_calls_named, nested_class};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MaxComplexitySchema;

impl Cop for MaxComplexitySchema {
    fn name(&self) -> &'static str {
        "GraphQL/MaxComplexitySchema"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/graphql/**/*_schema.rb"]
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
        let found = collect_calls_named(node, source, b"max_complexity")
            .into_iter()
            .any(|n| in_this_class(n, node));
        if found {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "max_complexity should be configured for schema.".into(),
        ));
    }
}

fn in_this_class(call: Node<'_>, class_node: Node<'_>) -> bool {
    let mut p = call.parent();
    while let Some(n) = p {
        if n.kind() == "class" {
            return n.id() == class_node.id();
        }
        p = n.parent();
    }
    false
}
