//! RSpec/InstanceVariable — prefer `let` over instance variables in specs.

use tree_sitter::Node;

use crate::cop::rspec::helpers::node_in_top_level_group;
use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct InstanceVariable;

const MSG: &str =
    "Avoid instance variables - use let, a method call, or a local variable (if possible).";

fn is_assign_lhs(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    // RuboCop searches `ivar` (reads), not `ivasgn` / `or_asgn` (assignments).
    if !matches!(parent.kind(), "assignment" | "operator_assignment") {
        return false;
    }
    parent
        .child_by_field_name("left")
        .is_some_and(|l| l.id() == node.id())
}

/// RuboCop `valid_usage?` — `Class.new { ... }` and custom matcher blocks.
fn valid_usage(source: &SourceFile, node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(cur) = p {
        if matches!(cur.kind(), "do_block" | "block") {
            if let Some(call) = cur.parent() {
                if is_class_new(source, call) || is_custom_matcher(source, call) {
                    return true;
                }
            }
        }
        p = cur.parent();
    }
    false
}

fn is_class_new(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(node.kind(), "call" | "command")
        && call_method_name(source, node) == Some(b"new")
        && call_receiver(node)
            .is_some_and(|r| r.kind() == "constant" && node_bytes(source, r) == b"Class")
}

fn first_arg_is_symbol(node: Node<'_>) -> bool {
    argument_nodes(node)
        .first()
        .is_some_and(|a| matches!(a.kind(), "simple_symbol" | "symbol"))
}

fn is_rspec_matchers(source: &SourceFile, node: Node<'_>) -> bool {
    node.kind() == "scope_resolution"
        && node
            .child_by_field_name("name")
            .is_some_and(|n| node_bytes(source, n) == b"Matchers")
        && node
            .child_by_field_name("scope")
            .is_some_and(|s| s.kind() == "constant" && node_bytes(source, s) == b"RSpec")
}

/// `matcher :name do` or `RSpec::Matchers.define :name do` (symbol arg required).
fn is_custom_matcher(source: &SourceFile, node: Node<'_>) -> bool {
    if !matches!(node.kind(), "call" | "command") || !first_arg_is_symbol(node) {
        return false;
    }
    match call_method_name(source, node) {
        Some(b"matcher") => call_receiver(node).is_none(),
        Some(b"define") => call_receiver(node).is_some_and(|r| is_rspec_matchers(source, r)),
        _ => false,
    }
}

impl Cop for InstanceVariable {
    fn name(&self) -> &'static str {
        "RSpec/InstanceVariable"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["instance_variable"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.kind() != "instance_variable"
            || is_assign_lhs(node)
            || !node_in_top_level_group(source, node)
            || valid_usage(source, node)
        {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(InstanceVariable, "cops/rspec/instance_variable");
}
