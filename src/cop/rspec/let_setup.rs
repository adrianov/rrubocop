//! RSpec/LetSetup — unused `let!` used only for side effects.

use std::collections::HashSet;

use tree_sitter::Node;

use crate::cop::rspec::helpers::{
    bare_rspec_call, block_body, call_block, first_sym_arg, is_group, RSPEC_INCLUDE,
};
use crate::cop::shared::{call_method_name, call_receiver, for_each_descendant, method_node, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct LetSetup;

const MSG: &str = "Do not use `let!` to setup objects not referenced in tests.";

fn skip_helper_name(name: &[u8]) -> bool {
    matches!(name, b"let" | b"let!" | b"subject" | b"subject!" | b"expect")
}

/// Ruby 3.1 `foo(bar:)` — RuboCop treats the implicit value as `(send nil? bar)`.
fn shorthand_kwarg_name(source: &SourceFile, node: Node<'_>) -> Option<Vec<u8>> {
    if node.kind() != "hash_key_symbol" {
        return None;
    }
    let parent = node.parent()?;
    if parent.kind() != "pair" || parent.child_by_field_name("value").is_some() {
        return None;
    }
    let raw = node_bytes(source, node);
    Some(raw.strip_suffix(b":").unwrap_or(raw).to_vec())
}

fn referenced_name(source: &SourceFile, node: Node<'_>) -> Option<Vec<u8>> {
    if let Some(n) = shorthand_kwarg_name(source, node) {
        return Some(n);
    }
    match node.kind() {
        "call" | "command" if call_receiver(node).is_none() => {
            call_method_name(source, node).map(|n| n.to_vec())
        }
        "identifier" => Some(node_bytes(source, node).to_vec()),
        _ => None,
    }
}

fn collect_used_names(source: &SourceFile, body: Node<'_>) -> HashSet<Vec<u8>> {
    let mut used = HashSet::new();
    for_each_descendant(body, |node| {
        if let Some(n) = referenced_name(source, node).filter(|n| !skip_helper_name(n)) {
            used.insert(n);
        }
    });
    used
}

fn each_let_bang<'a>(source: &SourceFile, body: Node<'a>, mut f: impl FnMut(Vec<u8>, Node<'a>)) {
    let mut cur = body.walk();
    for stmt in body.named_children(&mut cur) {
        if !matches!(stmt.kind(), "call" | "command") {
            continue;
        }
        if call_method_name(source, stmt) != Some(b"let!") {
            continue;
        }
        if let Some(name) = first_sym_arg(source, stmt) {
            f(name.to_vec(), stmt);
        }
    }
}

fn check_group(
    cop: &LetSetup,
    source: &SourceFile,
    body: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let used = collect_used_names(source, body);
    each_let_bang(source, body, |name, node| {
        if used.contains(&name) {
            return;
        }
        let meth = method_node(node).unwrap_or(node);
        let (line, col) = source.offset_to_line_col(meth.start_byte());
        diagnostics.push(cop.diagnostic(source, line, col, MSG.into()));
    });
}

impl Cop for LetSetup {
    fn name(&self) -> &'static str {
        "RSpec/LetSetup"
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
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(method) = bare_rspec_call(source, node) else {
            return;
        };
        if !is_group(method) {
            return;
        }
        let Some(body) = call_block(node).and_then(block_body) else {
            return;
        };
        check_group(self, source, body, diagnostics);
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(LetSetup, "cops/rspec/let_setup");
}
