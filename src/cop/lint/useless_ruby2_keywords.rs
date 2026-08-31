use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/UselessRuby2Keywords — ruby2_keywords on methods without **.
pub struct UselessRuby2Keywords;

fn has_kwrest(params: Node<'_>) -> bool {
    let mut cur = params.walk();
    params
        .named_children(&mut cur)
        .any(|n| matches!(n.kind(), "hash_splat_parameter" | "keyword_rest_parameter"))
}

fn method_arg(node: Node<'_>) -> Option<Node<'_>> {
    let arg = argument_nodes(node).into_iter().next()?;
    matches!(arg.kind(), "method" | "singleton_method").then_some(arg)
}

impl Cop for UselessRuby2Keywords {
    fn name(&self) -> &'static str {
        "Lint/UselessRuby2Keywords"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if call_method_name(source, node) != Some(b"ruby2_keywords") {
            return;
        }
        let Some(method) = method_arg(node) else {
            return;
        };
        // RuboCop: useless when no **kwargs (hash splat)
        if method.child_by_field_name("parameters").is_some_and(has_kwrest) {
            return;
        }
        let name = method
            .child_by_field_name("name")
            .map(|n| node_text(source, n))
            .unwrap_or_else(|| "unknown".into());
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("`ruby2_keywords` is unnecessary for method `{name}`."),
        ));
    }
}
