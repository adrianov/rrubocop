//! Style/MethodCallWithArgsParentheses — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_method_call_with_args_parentheses;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MethodCallWithArgsParentheses;

impl Cop for MethodCallWithArgsParentheses {
    fn name(&self) -> &'static str {
        "Style/MethodCallWithArgsParentheses"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_method_call_with_args_parentheses(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/MethodCallWithArgsParentheses offense.".to_string(),
        ));
    }
}
