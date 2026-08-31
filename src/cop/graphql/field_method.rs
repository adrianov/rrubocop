//! GraphQL/FieldMethod — prefer method: over trivial object.foo resolvers.

use tree_sitter::Node;

use super::helpers::{
    enclosing_class, find_method_def, has_kwarg, is_conflict_field_name, is_field_call,
    resolver_method_name, CALL_KINDS, DEPT_INCLUDE,
};
use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, named_kids, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FieldMethod;

impl Cop for FieldMethod {
    fn name(&self) -> &'static str {
        "GraphQL/FieldMethod"
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
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_field_call(source, node) || has_kwarg(source, node, "method") {
            return;
        }
        let Some(class) = enclosing_class(node) else {
            return;
        };
        let method_name = resolver_method_name(source, node);
        let Some(method) = find_method_def(class, source, &method_name) else {
            return;
        };
        let Some(suggested) = method_from_body(source, method) else {
            return;
        };
        if is_conflict_field_name(&suggested) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Use method: :{suggested}"),
        ));
    }
}

fn method_from_body(source: &SourceFile, method: Node<'_>) -> Option<String> {
    let expr = sole_body_expr(method)?;
    if !matches!(expr.kind(), "call" | "command" | "command_call") {
        return None;
    }
    let recv = call_receiver(expr)?;
    if node_text(source, recv) != "object" || !argument_nodes(expr).is_empty() {
        return None;
    }
    call_method_name(source, expr).map(|n| String::from_utf8_lossy(n).into_owned())
}

fn sole_body_expr(method: Node<'_>) -> Option<Node<'_>> {
    let body = method.child_by_field_name("body")?;
    let stmts = if body.kind() == "body_statement" {
        named_kids(body)
    } else {
        vec![body]
    };
    (stmts.len() == 1).then_some(stmts[0])
}
