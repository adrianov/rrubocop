use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/EachWithObjectArgument — immutable arg to each_with_object.
pub struct EachWithObjectArgument;

fn is_immutable(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "integer" | "float" | "simple_symbol" | "true" | "false" | "nil" | "string"
    )
}

impl Cop for EachWithObjectArgument {
    fn name(&self) -> &'static str {
        "Lint/EachWithObjectArgument"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"each_with_object"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if call_method_name(source, node) != Some(b"each_with_object") {
            return;
        }
        let args = argument_nodes(node);
        let Some(first) = args.first() else {
            return;
        };
        if !is_immutable(*first) {
            return;
        }
        let (line, col) = source.offset_to_line_col(first.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "The argument to each_with_object cannot be immutable.".to_string(),
        ));
    }
}
