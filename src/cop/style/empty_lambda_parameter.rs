//! Style/EmptyLambdaParameter — omit `()` on empty stabby lambdas.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLambdaParameter;

fn empty_parens_params(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(params) = node.child_by_field_name("parameters") else {
        return false;
    };
    node_bytes(source, params) == b"()"
}

impl Cop for EmptyLambdaParameter {
    fn name(&self) -> &'static str {
        "Style/EmptyLambdaParameter"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["lambda"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !empty_parens_params(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Omit parentheses for the empty lambda parameters.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(EmptyLambdaParameter, "cops/style/empty_lambda_parameter");
}
