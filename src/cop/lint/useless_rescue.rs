//! Lint/UselessRescue — rescue that only re-raises the same exception.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, for_each_descendant, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct UselessRescue;

fn is_raise_call(source: &SourceFile, node: Node<'_>) -> bool {
    match node.kind() {
        "identifier" => node_bytes(source, node) == b"raise",
        "call" | "command" => {
            call_method_name(source, node) == Some(b"raise")
                && node.child_by_field_name("receiver").is_none()
        }
        _ => false,
    }
}

fn rescue_body(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("body").or_else(|| {
        let mut cur = node.walk();
        node.named_children(&mut cur)
            .find(|n| matches!(n.kind(), "then" | "body_statement"))
    })
}

fn exception_variable<'a>(source: &'a SourceFile, rescue: Node<'_>) -> Option<&'a [u8]> {
    // tree-sitter-ruby: field `variable` → node kind `exception_variable`
    let ev = rescue
        .child_by_field_name("variable")
        .or_else(|| {
            let mut cur = rescue.walk();
            rescue
                .named_children(&mut cur)
                .find(|n| n.kind() == "exception_variable")
        })?;
    let mut cur = ev.walk();
    ev.named_children(&mut cur)
        .find(|n| n.kind() == "identifier")
        .map(|n| node_bytes(source, n))
}

fn only_reraising(source: &SourceFile, rescue: Node<'_>) -> bool {
    let Some(body) = rescue_body(rescue) else {
        return false;
    };
    let mut cur = body.walk();
    let stmts: Vec<_> = body
        .named_children(&mut cur)
        .filter(|n| n.kind() != "comment")
        .collect();
    if stmts.len() != 1 {
        return false;
    }
    is_reraise_expression(source, stmts[0], exception_variable(source, rescue))
}

fn is_reraise_expression(
    source: &SourceFile,
    node: Node<'_>,
    rescue_var: Option<&[u8]>,
) -> bool {
    if !is_raise_call(source, node) {
        return false;
    }
    let args = argument_nodes(node);
    if args.is_empty() {
        return true;
    }
    if args.len() > 1 {
        return false;
    }
    let text = node_bytes(source, args[0]);
    if text == b"$!" || text == b"$ERROR_INFO" {
        return true;
    }
    rescue_var.is_some_and(|v| v == text)
}

fn is_last_rescue(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return true;
    };
    let mut cur = parent.walk();
    let rescues: Vec<_> = parent
        .named_children(&mut cur)
        .filter(|n| n.kind() == "rescue")
        .collect();
    rescues.last().is_some_and(|n| n.id() == node.id())
}

fn exception_var_used_in_ensure(source: &SourceFile, rescue: Node<'_>) -> bool {
    let Some(var) = exception_variable(source, rescue) else {
        return false;
    };
    let Some(parent) = rescue.parent() else {
        return false;
    };
    let mut cur = parent.walk();
    let Some(ensure) = parent
        .named_children(&mut cur)
        .find(|n| n.kind() == "ensure")
    else {
        return false;
    };
    let mut used = false;
    for_each_descendant(ensure, |n| {
        if n.kind() == "identifier" && node_bytes(source, n) == var {
            used = true;
        }
    });
    used
}

fn modifier_only_reraising(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(handler) = node.child_by_field_name("handler") else {
        return false;
    };
    is_reraise_expression(source, handler, None)
}

fn rescue_modifier_keyword_byte(node: Node<'_>) -> usize {
    let mut cur = node.walk();
    for ch in node.children(&mut cur) {
        if !ch.is_named() && ch.kind() == "rescue" {
            return ch.start_byte();
        }
    }
    node.start_byte()
}

impl Cop for UselessRescue {
    fn name(&self) -> &'static str {
        "Lint/UselessRescue"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["rescue", "rescue_modifier"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let (line, col) = match node.kind() {
            "rescue_modifier" => {
                if !modifier_only_reraising(source, node) {
                    return;
                }
                source.offset_to_line_col(rescue_modifier_keyword_byte(node))
            }
            "rescue" => {
                if !is_last_rescue(node)
                    || !only_reraising(source, node)
                    || exception_var_used_in_ensure(source, node)
                {
                    return;
                }
                source.offset_to_line_col(node.start_byte())
            }
            _ => return,
        };
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Useless `rescue` detected.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(UselessRescue, "cops/lint/useless_rescue");
}
