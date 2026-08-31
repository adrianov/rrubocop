//! GraphQL/PrepareMethod — prefer prepare: :method over lambdas/consts.

use tree_sitter::Node;

use super::helpers::{is_argument_call, kwarg_value, CALL_KINDS, DEPT_INCLUDE};
use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct PrepareMethod;

impl Cop for PrepareMethod {
    fn name(&self) -> &'static str {
        "GraphQL/PrepareMethod"
    }

    fn default_include(&self) -> &'static [&'static str] {
        DEPT_INCLUDE
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        CALL_KINDS
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_argument_call(source, node) {
            return;
        }
        let Some(prep) = kwarg_value(source, node, "prepare") else {
            return;
        };
        let style = config.get_str("EnforcedStyle", "");
        let (line, col) = source.offset_to_line_col(prep.start_byte());
        if is_prepare_lambda(source, prep) {
            diagnostics.push(self.diagnostic(source, line, col, lambda_msg(style).into()));
            return;
        }
        if let Some(msg) = style_mismatch_msg(source, prep, style) {
            diagnostics.push(self.diagnostic(source, line, col, msg));
        }
    }
}

fn is_prepare_lambda(source: &SourceFile, prep: Node<'_>) -> bool {
    matches!(
        prep.kind(),
        "block" | "do_block" | "lambda" | "constant" | "scope_resolution"
    ) || (prep.kind() == "call" && node_text(source, prep).contains("->"))
}

fn lambda_msg(style: &str) -> &'static str {
    match style {
        "symbol" => "Avoid using prepare lambdas, use prepare: :method_name instead.",
        "string" => "Avoid using prepare lambdas, use prepare: \"method_name\" instead.",
        _ => {
            "Avoid using prepare lambdas, use prepare: :method_name or prepare: \"method_name\" instead."
        }
    }
}

fn style_mismatch_msg(source: &SourceFile, prep: Node<'_>, style: &str) -> Option<String> {
    let kind = prep.kind();
    if style == "symbol" && matches!(kind, "string" | "string_content" | "interpolated_string") {
        let name = node_text(source, prep)
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        Some(format!("Avoid using prepare string, use prepare: :{name} instead."))
    } else if style == "string" && matches!(kind, "simple_symbol" | "symbol" | "hash_key_symbol") {
        let name = node_text(source, prep).trim_start_matches(':').to_string();
        Some(format!(
            "Avoid using prepare symbols, use prepare: \"{name}\" instead."
        ))
    } else {
        None
    }
}
