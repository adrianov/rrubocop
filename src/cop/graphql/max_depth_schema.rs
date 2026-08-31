//! GraphQL/MaxDepthSchema — schema must set max_depth.

use tree_sitter::Node;

use super::helpers::{collect_calls_named, nested_class};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MaxDepthSchema;

impl Cop for MaxDepthSchema {
    fn name(&self) -> &'static str {
        "GraphQL/MaxDepthSchema"
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
        let found = collect_calls_named(node, source, b"max_depth")
            .into_iter()
            .any(|n| {
                let mut p = n.parent();
                while let Some(x) = p {
                    if x.kind() == "class" {
                        return x.id() == node.id();
                    }
                    p = x.parent();
                }
                false
            });
        if found {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "max_depth should be configured for schema.".into(),
        ));
    }
}
