//! Style/StabbyLambdaParentheses — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_stabby_lambda_parentheses;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct StabbyLambdaParentheses;

impl Cop for StabbyLambdaParentheses {
    fn name(&self) -> &'static str {
        "Style/StabbyLambdaParentheses"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["lambda"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_stabby_lambda_parentheses(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/StabbyLambdaParentheses offense.".to_string(),
        ));
    }
}
