//! RSpec/MultipleExpectations — limit expects per example.

use tree_sitter::Node;

use crate::cop::rspec::helpers::{
    bare_rspec_call, block_body, call_block, is_example, is_group, RSPEC_INCLUDE,
};
use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, method_node, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultipleExpectations;

const EXPECTS: &[&[u8]] = &[
    b"expect",
    b"is_expected",
    b"are_expected",
    b"expect_any_instance_of",
    b"should",
    b"should_not",
    b"should_receive",
    b"should_not_receive",
];

fn pair_af_value(source: &SourceFile, pair: Node<'_>) -> Option<bool> {
    let key = pair.child_by_field_name("key")?;
    let kb = node_bytes(source, key);
    let name = kb.strip_prefix(b":").unwrap_or(kb);
    if name != b"aggregate_failures" {
        return None;
    }
    Some(pair.child_by_field_name("value")?.kind() != "false")
}

fn af_from_arg(source: &SourceFile, arg: Node<'_>) -> Option<bool> {
    match arg.kind() {
        "simple_symbol" | "symbol" => {
            let b = node_bytes(source, arg);
            let name = b.strip_prefix(b":").unwrap_or(b);
            (name == b"aggregate_failures").then_some(true)
        }
        "pair" => pair_af_value(source, arg),
        "hash" | "bare_hash" => {
            let mut cur = arg.walk();
            arg.named_children(&mut cur)
                .find_map(|c| (c.kind() == "pair").then(|| pair_af_value(source, c)).flatten())
        }
        _ => None,
    }
}

/// `:aggregate_failures` / `aggregate_failures: true|false` on an example or group.
fn has_aggregate_failures_metadata(source: &SourceFile, node: Node<'_>) -> Option<bool> {
    argument_nodes(node)
        .into_iter()
        .find_map(|arg| af_from_arg(source, arg))
}

fn ancestor_aggregate_failures(source: &SourceFile, mut node: Node<'_>) -> bool {
    let mut inherited = false;
    let mut groups = Vec::new();
    while let Some(parent) = node.parent() {
        node = parent;
        if !matches!(node.kind(), "call" | "command") {
            continue;
        }
        if bare_rspec_call(source, node).is_some_and(is_group) {
            groups.push(node);
        }
    }
    for group in groups.into_iter().rev() {
        match has_aggregate_failures_metadata(source, group) {
            Some(v) => inherited = v,
            None => {}
        }
    }
    inherited
}

fn skips_multiple_expects(source: &SourceFile, example: Node<'_>) -> bool {
    match has_aggregate_failures_metadata(source, example) {
        Some(true) => true,
        Some(false) => false,
        None => ancestor_aggregate_failures(source, example),
    }
}

fn is_bare_method(source: &SourceFile, node: Node<'_>, want: &[u8]) -> bool {
    matches!(node.kind(), "call" | "command")
        && call_receiver(node).is_none()
        && call_method_name(source, node) == Some(want)
}

fn is_expect_call(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(node.kind(), "call" | "command")
        && call_receiver(node).is_none()
        && call_method_name(source, node).is_some_and(|m| EXPECTS.iter().any(|&e| e == m))
}

fn walk_expects(source: &SourceFile, node: Node<'_>, n: &mut usize) {
    if is_bare_method(source, node, b"aggregate_failures") && call_block(node).is_some() {
        *n += 1;
        return;
    }
    if is_expect_call(source, node) {
        *n += 1;
    }
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        walk_expects(source, child, n);
    }
}

fn count_expects(source: &SourceFile, body: Node<'_>) -> usize {
    let mut n = 0;
    walk_expects(source, body, &mut n);
    n
}

fn report_too_many_expects(
    cop: &MultipleExpectations,
    source: &SourceFile,
    node: Node<'_>,
    count: usize,
    max: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let meth = method_node(node).unwrap_or(node);
    let (line, col) = source.offset_to_line_col(meth.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!("Example has too many expectations [{count}/{max}]."),
    ));
}

impl Cop for MultipleExpectations {
    fn name(&self) -> &'static str {
        "RSpec/MultipleExpectations"
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_INCLUDE
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
        if bare_rspec_call(source, node).is_none_or(|m| !is_example(m)) {
            return;
        }
        if skips_multiple_expects(source, node) {
            return;
        }
        let Some(body) = call_block(node).and_then(block_body) else {
            return;
        };
        let max = config.get_usize("Max", 1);
        let count = count_expects(source, body);
        if count > max {
            report_too_many_expects(self, source, node, count, max, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(MultipleExpectations, "cops/rspec/multiple_expectations");
}
