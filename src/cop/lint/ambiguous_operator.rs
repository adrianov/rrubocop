use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/AmbiguousOperator — ambiguous splat/block/unary in command-style call.
pub struct AmbiguousOperator;

fn unary_ops(source: &SourceFile, first: Node<'_>) -> Option<(&'static str, &'static str, &'static str)> {
    let t = node_text(source, first);
    if t.starts_with('+') {
        Some(("positive number", "an addition", "+"))
    } else if t.starts_with('-') {
        Some(("negative number", "a subtraction", "-"))
    } else {
        None
    }
}

fn ambiguous_ops(
    source: &SourceFile,
    first: Node<'_>,
) -> Option<(&'static str, &'static str, &'static str)> {
    match first.kind() {
        "splat_argument" => Some(("splat", "a multiplication", "*")),
        "hash_splat_argument" | "hash_splat" => Some(("keyword splat", "an exponent", "**")),
        "block_argument" => Some(("block", "a binary AND", "&")),
        "unary" => unary_ops(source, first),
        _ => None,
    }
}

impl Cop for AmbiguousOperator {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousOperator"
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
        let Some(args_node) = node.child_by_field_name("arguments") else {
            return;
        };
        if node_bytes(source, args_node).starts_with(b"(") {
            return;
        }
        let Some(first) = argument_nodes(node).into_iter().next() else {
            return;
        };
        let Some((actual, possible, op)) = ambiguous_ops(source, first) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(first.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!(
                "Ambiguous {actual} operator. Parenthesize the method arguments if it's surely a {actual} operator, or add a whitespace to the right of the `{op}` if it should be {possible}."
            ),
        ));
    }
}
