//! GraphQL/GraphqlName — graphql_name required or only for overrides.

use tree_sitter::Node;

use super::helpers::{class_leaf_name, collect_calls_named, enclosing_class, nested_class};
use crate::cop::shared::{argument_nodes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct GraphqlName;

impl Cop for GraphqlName {
    fn name(&self) -> &'static str {
        "GraphQL/GraphqlName"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/graphql/types/**/*", "**/graphql/mutations/**/*"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if nested_class(node) {
            return;
        }
        if let Some(msg) = name_offense(source, node, config) {
            let (line, col) = source.offset_to_line_col(node.start_byte());
            diagnostics.push(self.diagnostic(source, line, col, msg.into()));
        }
    }
}

fn name_offense(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> Option<&'static str> {
    let specified = find_graphql_name(source, node);
    if config.get_str("EnforcedStyle", "only_override") == "required" {
        return specified.is_none().then_some("graphql_name should be configured.");
    }
    let spec = specified?;
    let leaf = class_leaf_name(source, node)?;
    let default_name = leaf.strip_suffix("Type").unwrap_or(&leaf);
    (spec == default_name).then_some("graphql_name should be specified only for overrides.")
}

fn find_graphql_name(source: &SourceFile, class_node: Node<'_>) -> Option<String> {
    let n = collect_calls_named(class_node, source, b"graphql_name")
        .into_iter()
        .find(|n| in_own_class(*n, class_node))?;
    let a = *argument_nodes(n).first()?;
    Some(
        node_text(source, a)
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string(),
    )
}

fn in_own_class(n: Node<'_>, class_node: Node<'_>) -> bool {
    enclosing_class(n).is_some_and(|c| c.id() == class_node.id())
}
