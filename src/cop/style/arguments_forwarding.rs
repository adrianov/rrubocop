//! Style/ArgumentsForwarding — use `...` for forwarded args (Ruby 2.7+).

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ArgumentsForwarding;

fn param_name<'a>(source: &'a SourceFile, node: Node<'a>) -> Option<&'a [u8]> {
    node.child_by_field_name("name")
        .map(|n| node_bytes(source, n))
        .or_else(|| {
            let mut cur = node.walk();
            node.named_children(&mut cur)
                .find(|n| n.kind() == "identifier")
                .map(|n| node_bytes(source, n))
        })
}

fn rest_and_block<'a>(source: &'a SourceFile, params: Node<'a>) -> Option<(&'a [u8], &'a [u8])> {
    let mut rest = None;
    let mut block = None;
    let mut cur = params.walk();
    for child in params.named_children(&mut cur) {
        match child.kind() {
            "splat_parameter" => rest = param_name(source, child),
            "block_parameter" => block = param_name(source, child),
            _ => {}
        }
    }
    Some((rest?, block?))
}

fn arg_ident<'a>(source: &'a SourceFile, node: Node<'a>) -> Option<&'a [u8]> {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .find(|n| n.kind() == "identifier")
        .map(|n| node_bytes(source, n))
}

fn forwards_call(source: &SourceFile, node: Node<'_>, rest: &[u8], block: &[u8]) -> bool {
    if node.kind() != "call" && node.kind() != "command" {
        return false;
    }
    let args = argument_nodes(node);
    let mut has_rest = false;
    let mut has_block = false;
    for arg in &args {
        match arg.kind() {
            "splat_argument" if arg_ident(source, *arg) == Some(rest) => has_rest = true,
            "block_argument" if arg_ident(source, *arg) == Some(block) => has_block = true,
            _ => {}
        }
    }
    has_rest && has_block
}

fn lvars_in_range(body: Node<'_>, start: usize, end: usize) -> bool {
    let mut ok = true;
    crate::cop::shared::for_each_descendant(body, |n| {
        if !ok {
            return;
        }
        if n.kind() != "identifier" {
            return;
        }
        let p = n.parent();
        if p.is_some_and(|pr| {
            matches!(
                pr.kind(),
                "assignment"
                    | "operator_assignment"
                    | "method"
                    | "singleton_method"
                    | "optional_parameter"
                    | "keyword_parameter"
                    | "splat_parameter"
                    | "block_parameter"
                    | "hash_splat_parameter"
            )
        }) {
            return;
        }
        if !(start..end).contains(&n.start_byte()) {
            ok = false;
        }
    });
    ok
}

fn method_body(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("body").or_else(|| {
        let mut cur = node.walk();
        node.named_children(&mut cur)
            .find(|n| n.kind() == "body_statement")
    })
}

fn find_forwarding_call(
    source: &SourceFile,
    body: Node<'_>,
    rest: &[u8],
    block: &[u8],
) -> Option<usize> {
    let mut stack = vec![body];
    while let Some(n) = stack.pop() {
        if forwards_call(source, n, rest, block) && lvars_in_range(body, n.start_byte(), n.end_byte()) {
            return Some(n.start_byte());
        }
        let mut cur = n.walk();
        for child in n.named_children(&mut cur) {
            if matches!(child.kind(), "call" | "command" | "body_statement" | "begin") {
                stack.push(child);
            }
        }
    }
    None
}

impl Cop for ArgumentsForwarding {
    fn name(&self) -> &'static str {
        "Style/ArgumentsForwarding"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if config.get_f64("TargetRubyVersion", 2.7) < 2.7 {
            return;
        }
        let Some(params) = node.child_by_field_name("parameters") else {
            return;
        };
        let Some((rest, block)) = rest_and_block(source, params) else {
            return;
        };
        if !config.get_bool("AllowOnlyRestArgument", true) {
            return;
        }
        let Some(body) = method_body(node) else {
            return;
        };
        let Some(off) = find_forwarding_call(source, body, rest, block) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(off);
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Use arguments forwarding.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ArgumentsForwarding, "cops/style/arguments_forwarding");
}
