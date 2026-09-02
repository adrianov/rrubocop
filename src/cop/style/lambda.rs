//! Style/Lambda — `lambda` vs stabby `->` by line count or enforced style.

use tree_sitter::Node;

use crate::cop::shared::call_method_name;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct Lambda;

fn multiline(node: Node<'_>) -> bool {
    node.start_position().row != node.end_position().row
}

fn offending_stabby(style: &str, multi: bool) -> bool {
    match style {
        "literal" => true,
        "lambda" => true,
        _ => multi, // line_count_dependent
    }
}

fn offending_lambda_kw(style: &str, multi: bool) -> bool {
    match style {
        "literal" => false,
        "lambda" => false,
        _ => !multi, // line_count_dependent
    }
}

fn lambda_block(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("block")
        .or_else(|| node.child_by_field_name("body"))
}

fn check_stabby(
    cop: &Lambda,
    source: &SourceFile,
    node: Node<'_>,
    style: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let body = lambda_block(node).unwrap_or(node);
    if !offending_stabby(style, multiline(body)) {
        return;
    }
    let msg = format!(
        "Use the `lambda` method for {} lambdas.",
        if multiline(body) { "multiline" } else { "all" }
    );
    let (line, col) = source.offset_to_line_col(node.start_byte());
    diagnostics.push(cop.diagnostic(source, line, col, msg));
}

fn check_lambda_kw(
    cop: &Lambda,
    source: &SourceFile,
    node: Node<'_>,
    style: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(block) = lambda_block(node) else {
        return;
    };
    if !offending_lambda_kw(style, multiline(block)) {
        return;
    }
    let msg = format!(
        "Use the `-> {{ ... }}` lambda literal syntax for {} lambdas.",
        if multiline(block) { "all" } else { "single line" }
    );
    let (line, col) = source.offset_to_line_col(node.start_byte());
    diagnostics.push(cop.diagnostic(source, line, col, msg));
}

impl Cop for Lambda {
    fn name(&self) -> &'static str {
        "Style/Lambda"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["lambda", "call", "command"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"lambda"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "line_count_dependent");
        if node.kind() == "lambda" {
            check_stabby(self, source, node, style, diagnostics);
            return;
        }
        if call_method_name(source, node) != Some(b"lambda") {
            return;
        }
        check_lambda_kw(self, source, node, style, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(Lambda, "cops/style/lambda");
}
