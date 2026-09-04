//! RSpec/IteratedExpectation — prefer `all` matcher over `.each` + `expect`.

use tree_sitter::Node;

use crate::cop::rspec::helpers::{block_body, RSPEC_INCLUDE};
use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct IteratedExpectation;

const MSG: &str = "Prefer using the `all` matcher instead of iterating over an array.";

fn each_call<'a>(source: &SourceFile, block: Node<'a>) -> Option<Node<'a>> {
    let call = block.parent()?;
    if !matches!(call.kind(), "call" | "command" | "command_call") {
        return None;
    }
    (call_method_name(source, call) == Some(b"each")).then_some(call)
}

/// Single block arg, or `_1` when parameters are omitted (RuboCop numblock).
fn block_arg_name<'a>(source: &'a SourceFile, block: Node<'_>) -> Option<&'a [u8]> {
    let Some(params) = block.child_by_field_name("parameters") else {
        return Some(b"_1");
    };
    let mut cur = params.walk();
    let ids: Vec<_> = params
        .named_children(&mut cur)
        .filter(|n| n.kind() == "identifier")
        .collect();
    (ids.len() == 1).then(|| node_bytes(source, ids[0]))
}

/// RuboCop `expectation?`: `expect(arg).to …` (not `not_to`).
fn is_expectation(source: &SourceFile, node: Node<'_>, arg: &[u8]) -> bool {
    if call_method_name(source, node) != Some(b"to") {
        return false;
    }
    let Some(recv) = call_receiver(node) else {
        return false;
    };
    if call_method_name(source, recv) != Some(b"expect") {
        return false;
    }
    argument_nodes(recv)
        .into_iter()
        .next()
        .is_some_and(|a| a.kind() == "identifier" && node_bytes(source, a) == arg)
}

fn body_nodes(body: Node<'_>) -> Vec<Node<'_>> {
    match body.kind() {
        "body_statement" | "block_body" => {
            let mut cur = body.walk();
            body.named_children(&mut cur)
                .filter(|n| n.kind() != "comment")
                .collect()
        }
        _ => vec![body],
    }
}

fn only_expectations(source: &SourceFile, body: Node<'_>, arg: &[u8]) -> bool {
    let nodes = body_nodes(body);
    !nodes.is_empty() && nodes.iter().all(|n| is_expectation(source, *n, arg))
}

impl Cop for IteratedExpectation {
    fn name(&self) -> &'static str {
        "RSpec/IteratedExpectation"
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_INCLUDE
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["do_block", "block"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(call) = each_call(source, node) else {
            return;
        };
        let Some(arg) = block_arg_name(source, node) else {
            return;
        };
        let Some(body) = block_body(node) else {
            return;
        };
        if !only_expectations(source, body, arg) {
            return;
        }
        let (line, col) = source.offset_to_line_col(call.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.into()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(IteratedExpectation, "cops/rspec/iterated_expectation");
}
