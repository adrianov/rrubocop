//! Rails/HelperInstanceVariable — no ivars in helpers (outside nested classes).

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct HelperInstanceVariable;

const MSG: &str = "Do not use instance variables in helpers.";

fn is_or_eq_assign(source: &SourceFile, node: Node<'_>) -> bool {
    let op = &source.as_bytes()[node.start_byte()..node.end_byte()];
    op.windows(3).any(|w| w == b"||=")
}

fn assigned_ivar<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    let left = node.child_by_field_name("left")?;
    if left.kind() != "instance_variable" {
        return None;
    }
    if node.kind() == "operator_assignment" && is_or_eq_assign(source, node) {
        return None;
    }
    Some(left)
}

fn target_ivar<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    match node.kind() {
        "instance_variable" => Some(node),
        "assignment" | "operator_assignment" => assigned_ivar(source, node),
        _ => None,
    }
}

fn nested_in_class(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(cur) = p {
        if cur.kind() == "class" {
            return true;
        }
        p = cur.parent();
    }
    false
}

impl Cop for HelperInstanceVariable {
    fn name(&self) -> &'static str {
        "Rails/HelperInstanceVariable"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["app/helpers/**/*.rb"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["instance_variable", "assignment", "operator_assignment"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(ivar) = target_ivar(source, node) else {
            return;
        };
        if nested_in_class(node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(ivar.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
