//! GraphQL/ResolverMethodLength — resolver methods must stay short.

use tree_sitter::Node;

use super::helpers::{
    class_body_stmts, enclosing_class, field_name, is_field_call, method_line_count, DEPT_INCLUDE,
};
use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ResolverMethodLength;

impl Cop for ResolverMethodLength {
    fn name(&self) -> &'static str {
        "GraphQL/ResolverMethodLength"
    }

    fn default_include(&self) -> &'static [&'static str] {
        DEPT_INCLUDE
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some((length, max)) = over_limit(source, node, config) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("ResolverMethod has too many lines. [{length}/{max}]"),
        ));
    }
}

fn over_limit(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> Option<(usize, usize)> {
    let name = node_text(source, node.child_by_field_name("name")?);
    if excluded(config, &name) {
        return None;
    }
    let class = enclosing_class(node)?;
    if !resolves_field(source, class, &name) {
        return None;
    }
    let max = config.get_usize("Max", 10);
    let length = method_line_count(node).saturating_sub(2);
    (length > max).then_some((length, max))
}

fn excluded(config: &CopConfig, name: &str) -> bool {
    super::helpers::config_string_list(config, "ExcludedMethods")
        .iter()
        .any(|e| e == name)
}

fn resolves_field(source: &SourceFile, class: Node<'_>, name: &str) -> bool {
    class_body_stmts(class)
        .into_iter()
        .filter(|n| is_field_call(source, *n))
        .filter_map(|n| field_name(source, n))
        .any(|f| f == name)
}
