use tree_sitter::Node;

use crate::cop::shared::{for_each_descendant, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/MissingSuper — initialize/callback without super.
pub struct MissingSuper;

const CALLBACKS: &[&[u8]] = &[
    b"initialize",
    b"method_missing",
    b"respond_to_missing?",
];

fn enclosing_class(node: Node<'_>) -> Option<Node<'_>> {
    let mut parent = node.parent();
    while let Some(p) = parent {
        if p.kind() == "class" {
            return Some(p);
        }
        if matches!(p.kind(), "module" | "singleton_class") {
            return None;
        }
        parent = p.parent();
    }
    None
}

fn has_super_call(source: &SourceFile, body: Node<'_>) -> bool {
    let mut found = false;
    for_each_descendant(body, |n| {
        if n.kind() == "super" {
            found = true;
        } else if n.kind() == "identifier" && node_bytes(source, n) == b"super" {
            found = true;
        } else if n.kind() == "call" {
            if let Some(m) = n.child_by_field_name("method") {
                if node_bytes(source, m) == b"super" {
                    found = true;
                }
            }
        }
    });
    found
}

fn missing_msg(name: &[u8]) -> &'static str {
    if name == b"initialize" {
        "Call `super` to initialize state of the parent class."
    } else {
        "Call `super` to invoke callback defined in the parent class."
    }
}

impl Cop for MissingSuper {
    fn name(&self) -> &'static str {
        "Lint/MissingSuper"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(class_node) = enclosing_class(node) else {
            return;
        };
        if class_node.child_by_field_name("superclass").is_none() {
            return;
        }
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        let name_b = node_bytes(source, name);
        if !CALLBACKS.contains(&name_b) {
            return;
        }
        if let Some(body) = node.child_by_field_name("body") {
            if has_super_call(source, body) {
                return;
            }
        }
        let (line, col) = source.offset_to_line_col(name.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, missing_msg(name_b).to_string()));
    }
}
