//! Rails/Output — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, method_node, node_bytes, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct Output;

const METHODS: &[&[u8]] = &[b"puts", b"print", b"p", b"pp", b"pretty_print", b"ap"];
/// Bare `p` is indistinguishable from a local/param in tree-sitter; skip it.
const BARE_METHODS: &[&[u8]] = &[b"puts", b"print", b"pp", b"pretty_print", b"ap"];

fn under_method_arg(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        "call" | "command" | "command_call" => true,
        "argument_list" => parent
            .parent()
            .is_some_and(|gp| matches!(gp.kind(), "call" | "command" | "command_call")),
        _ => false,
    }
}

fn call_stdout_span(source: &SourceFile, node: Node<'_>) -> Option<(usize, usize)> {
    let method = call_method_name(source, node)?;
    if !METHODS.contains(&method) || call_receiver(node).is_some() {
        return None;
    }
    // RuboCop: skip send whose parent is another call (`foo(puts)`, `puts.bar`).
    // `next puts …` has argument_list under `next` — still an offense.
    if under_method_arg(node) {
        return None;
    }
    let meth = method_node(node).unwrap_or(node);
    Some((meth.start_byte(), meth.end_byte()))
}

fn is_assignment_target(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        "assignment" | "operator_assignment" => parent
            .child_by_field_name("left")
            .is_some_and(|left| left == node),
        "left_assignment_list" | "rest_assignment" | "destructured_left_assignment" => true,
        _ => false,
    }
}

fn lhs_binds_name(source: &SourceFile, left: Node<'_>, name: &[u8]) -> bool {
    match left.kind() {
        "identifier" => node_bytes(source, left) == name,
        "left_assignment_list" | "rest_assignment" | "destructured_left_assignment" => {
            let mut cur = left.walk();
            left.named_children(&mut cur)
                .any(|c| lhs_binds_name(source, c, name))
        }
        _ => false,
    }
}

fn enclosing_method(node: Node<'_>) -> Option<Node<'_>> {
    let mut cur = node.parent();
    while let Some(n) = cur {
        if matches!(n.kind(), "method" | "singleton_method") {
            return Some(n);
        }
        cur = n.parent();
    }
    None
}

fn assignment_scope(node: Node<'_>) -> Option<Node<'_>> {
    enclosing_method(node).or_else(|| {
        let mut cur = Some(node);
        while let Some(n) = cur {
            if n.kind() == "program" {
                return Some(n);
            }
            cur = n.parent();
        }
        None
    })
}

fn scope_walk_roots(scope: Node<'_>) -> Vec<Node<'_>> {
    if scope.kind() == "program" {
        let mut cur = scope.walk();
        return scope.named_children(&mut cur).collect();
    }
    scope.child_by_field_name("body").into_iter().collect()
}

fn node_assigns_name(source: &SourceFile, n: Node<'_>, name: &[u8]) -> bool {
    matches!(n.kind(), "assignment" | "operator_assignment")
        && n.child_by_field_name("left")
            .is_some_and(|left| lhs_binds_name(source, left, name))
}

fn name_assigned_in_scope(source: &SourceFile, name: &[u8], node: Node<'_>) -> bool {
    let Some(scope) = assignment_scope(node) else {
        return false;
    };
    let mut stack = scope_walk_roots(scope);
    while let Some(n) = stack.pop() {
        if matches!(n.kind(), "method" | "singleton_method") {
            continue;
        }
        if node_assigns_name(source, n, name) {
            return true;
        }
        let mut cur = n.walk();
        stack.extend(n.named_children(&mut cur));
    }
    false
}

fn params_bind_name(source: &SourceFile, params: Node<'_>, name: &[u8]) -> bool {
    let mut stack = vec![params];
    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" && node_bytes(source, n) == name {
            return true;
        }
        let mut cur = n.walk();
        stack.extend(n.named_children(&mut cur));
    }
    false
}

fn block_or_lambda_binds(source: &SourceFile, n: Node<'_>, name: &[u8]) -> bool {
    let want = match n.kind() {
        "block" | "do_block" => "block_parameters",
        "lambda" => "lambda_parameters",
        _ => return false,
    };
    let mut cur = n.walk();
    n.named_children(&mut cur)
        .any(|ch| ch.kind() == want && params_bind_name(source, ch, name))
}

fn name_bound_by_params(source: &SourceFile, name: &[u8], node: Node<'_>) -> bool {
    let mut cur = node.parent();
    while let Some(n) = cur {
        if matches!(n.kind(), "method" | "singleton_method") {
            return n
                .child_by_field_name("parameters")
                .is_some_and(|p| params_bind_name(source, p, name));
        }
        if block_or_lambda_binds(source, n, name) {
            return true;
        }
        cur = n.parent();
    }
    false
}

fn bare_binding_or_call_site(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    matches!(
        parent.kind(),
        "call"
            | "command"
            | "command_call"
            | "argument_list"
            | "method"
            | "singleton_method"
            | "parameters"
            | "method_parameters"
            | "bare_parameters"
            | "block_parameters"
            | "lambda_parameters"
            | "destructured_parameter"
    )
}

fn name_is_local_binding(source: &SourceFile, name: &[u8], node: Node<'_>) -> bool {
    name_assigned_in_scope(source, name, node) || name_bound_by_params(source, name, node)
}

fn bare_call_identifier(source: &SourceFile, node: Node<'_>) -> bool {
    if is_assignment_target(node) || bare_binding_or_call_site(node) {
        return false;
    }
    !name_is_local_binding(source, node_bytes(source, node), node)
}

fn bare_stdout_span(source: &SourceFile, node: Node<'_>) -> Option<(usize, usize)> {
    if !BARE_METHODS.contains(&node_bytes(source, node)) || !bare_call_identifier(source, node) {
        return None;
    }
    Some((node.start_byte(), node.end_byte()))
}

fn stdout_span(source: &SourceFile, node: Node<'_>) -> Option<(usize, usize)> {
    match node.kind() {
        "call" | "command" | "command_call" => call_stdout_span(source, node),
        // Bare `puts` / `print` with no args parse as `identifier`, not `call`.
        "identifier" => bare_stdout_span(source, node),
        _ => None,
    }
}

impl Cop for Output {
    fn name(&self) -> &'static str {
        "Rails/Output"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &[
            "**/app/**/*.rb",
            "**/config/**/*.rb",
            "db/**/*.rb",
            "**/lib/**/*.rb",
        ]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn safe_autocorrect(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "identifier"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some((start, end)) = stdout_span(source, node) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(start);
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            "Do not write to stdout. Use Rails's logger if you want to log.".to_string(),
        );
        if push_replace(
            &mut corrections,
            start,
            end,
            "Rails.logger.debug",
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(Output, "cops/rails/output");
}
