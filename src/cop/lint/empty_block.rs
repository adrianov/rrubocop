//! Lint/EmptyBlock — blocks with no body.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, is_const_named, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct EmptyBlock;

fn body_empty(node: Node<'_>) -> bool {
    if let Some(body) = node.child_by_field_name("body") {
        let mut cur = body.walk();
        return body
            .named_children(&mut cur)
            .all(|c| c.kind() == "comment");
    }
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .all(|c| matches!(c.kind(), "block_parameters" | "comment"))
}

fn named_comment(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.named_children(&mut cur).any(|c| c.kind() == "comment")
}

fn root_of(mut node: Node<'_>) -> Node<'_> {
    while let Some(parent) = node.parent() {
        node = parent;
    }
    node
}

/// True if a real `comment` node overlaps the offense span through end-of-line
/// (covers trailing `# TODO` after `}` / `end`).
fn comment_on_span(node: Node<'_>, start: usize, line_end: usize) -> bool {
    let mut stack = vec![root_of(node)];
    while let Some(n) = stack.pop() {
        if n.kind() == "comment" {
            if n.start_byte() < line_end && n.end_byte() > start {
                return true;
            }
            continue;
        }
        let mut cur = n.walk();
        for child in n.named_children(&mut cur) {
            stack.push(child);
        }
    }
    false
}

fn has_comment(source: &SourceFile, node: Node<'_>) -> bool {
    if named_comment(node) || node.child_by_field_name("body").is_some_and(named_comment) {
        return true;
    }
    let bytes = source.as_bytes();
    let start = node.start_byte();
    let end = node.end_byte().min(bytes.len());
    comment_on_span(
        node,
        start,
        bytes[end..]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(bytes.len(), |i| end + i),
    )
}

fn is_proc_new(source: &SourceFile, call: Node<'_>) -> bool {
    call_method_name(source, call) == Some(b"new")
        && call_receiver(call).is_some_and(|r| {
            is_const_named(source, r, b"Proc")
                || (r.kind() == "scope_resolution"
                    && r.child_by_field_name("name")
                        .is_some_and(|n| node_bytes(source, n) == b"Proc"))
        })
}

fn lambda_or_proc(source: &SourceFile, block: Node<'_>) -> bool {
    let Some(parent) = block.parent() else {
        return false;
    };
    if parent.kind() == "lambda" {
        return true;
    }
    match call_method_name(source, parent) {
        Some(b"lambda" | b"proc") => call_receiver(parent).is_none(),
        Some(b"new") => is_proc_new(source, parent),
        _ => false,
    }
}

fn offense_node(block: Node<'_>) -> Node<'_> {
    block
        .parent()
        .filter(|p| matches!(p.kind(), "call" | "command" | "lambda"))
        .unwrap_or(block)
}

impl Cop for EmptyBlock {
    fn name(&self) -> &'static str {
        "Lint/EmptyBlock"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["block", "do_block"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !body_empty(node) {
            return;
        }
        if config.get_bool("AllowEmptyLambdas", true) && lambda_or_proc(source, node) {
            return;
        }
        let target = offense_node(node);
        if config.get_bool("AllowComments", true) && has_comment(source, target) {
            return;
        }
        let (line, col) = source.offset_to_line_col(target.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Empty block detected.".into(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(EmptyBlock, "cops/lint/empty_block");
}
