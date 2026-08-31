//! Lint/NestedMethodDefinition — method defined inside method.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, is_const_named};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct NestedMethodDefinition;

const EVAL_METHODS: &[&[u8]] = &[b"instance_eval", b"class_eval", b"module_eval"];
const EXEC_METHODS: &[&[u8]] = &[b"instance_exec", b"class_exec", b"module_exec"];
const CLASS_CTOR: &[&[u8]] = &[b"Class", b"Module", b"Struct"];

fn allowed_singleton_subject(node: Node<'_>) -> bool {
    if node.kind() != "singleton_method" {
        return false;
    }
    let Some(obj) = node.child_by_field_name("object") else {
        return false;
    };
    matches!(
        obj.kind(),
        "identifier"
            | "instance_variable"
            | "class_variable"
            | "global_variable"
            | "constant"
            | "call"
            | "parenthesized_statements"
    )
}

fn is_class_ctor_call(source: &SourceFile, call: Node<'_>) -> bool {
    let Some(method) = call_method_name(source, call) else {
        return false;
    };
    let Some(recv) = call_receiver(call) else {
        return false;
    };
    if method == b"define" {
        return is_const_named(source, recv, b"Data");
    }
    method == b"new" && CLASS_CTOR.iter().any(|&name| is_const_named(source, recv, name))
}

fn block_call<'a>(block: Node<'a>) -> Option<Node<'a>> {
    let parent = block.parent()?;
    match parent.kind() {
        "call" | "command" => Some(parent),
        _ => None,
    }
}

fn is_scoping_ancestor(source: &SourceFile, n: Node<'_>) -> bool {
    match n.kind() {
        "singleton_class" => true,
        "do_block" | "block" => {
            let Some(call) = block_call(n) else {
                return false;
            };
            if is_class_ctor_call(source, call) {
                return true;
            }
            let Some(method) = call_method_name(source, call) else {
                return false;
            };
            EVAL_METHODS.contains(&method)
                || EXEC_METHODS.contains(&method)
                || method == b"define_method"
        }
        _ => false,
    }
}

fn nested_without_scope(source: &SourceFile, node: Node<'_>) -> bool {
    let mut p = node.parent();
    let mut saw_scope = false;
    while let Some(n) = p {
        if is_scoping_ancestor(source, n) {
            saw_scope = true;
        }
        match n.kind() {
            "method" | "singleton_method" => return !saw_scope,
            "class" | "module" | "program" => return false,
            // singleton_class is scoping, keep walking for an outer method
            _ => {}
        }
        p = n.parent();
    }
    false
}

impl Cop for NestedMethodDefinition {
    fn name(&self) -> &'static str {
        "Lint/NestedMethodDefinition"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if allowed_singleton_subject(node) {
            return;
        }
        if !nested_without_scope(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Method definitions must not be nested. Use `lambda` instead.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(NestedMethodDefinition, "cops/lint/nested_method_definition");
}
