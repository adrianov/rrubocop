use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/RequireParentheses — predicate/ ternary-ish without parens + boolean arg.
pub struct RequireParentheses;

const MSG: &str =
    "Use parentheses in the method call to avoid confusion about precedence.";

fn is_bool_op(source: &SourceFile, node: Node<'_>) -> bool {
    if node.kind() != "binary" {
        return false;
    }
    let Some(op) = node.child_by_field_name("operator") else {
        return false;
    };
    matches!(node_bytes(source, op), b"&&" | b"||" | b"and" | b"or")
}

fn check_defined(source: &SourceFile, node: Node<'_>) -> bool {
    let text = node_text(source, node);
    if !text.starts_with("defined?") {
        return false;
    }
    node.child_by_field_name("operand")
        .is_some_and(|op| is_bool_op(source, op))
}

fn check_predicate(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(meth) = call_method_name(source, node) else {
        return false;
    };
    if !meth.ends_with(b"?") {
        return false;
    }
    let Some(args_node) = node.child_by_field_name("arguments") else {
        return false;
    };
    if node_bytes(source, args_node).starts_with(b"(") {
        return false;
    }
    argument_nodes(node)
        .first()
        .is_some_and(|first| is_bool_op(source, *first))
}

impl Cop for RequireParentheses {
    fn name(&self) -> &'static str {
        "Lint/RequireParentheses"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "unary"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let hit = if node.kind() == "unary" {
            check_defined(source, node)
        } else {
            check_predicate(source, node)
        };
        if !hit {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
