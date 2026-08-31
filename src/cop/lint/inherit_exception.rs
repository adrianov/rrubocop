use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/InheritException — class inheriting Exception.
pub struct InheritException;

fn exception_name(source: &SourceFile, node: Node<'_>) -> bool {
    match node.kind() {
        "constant" => node_bytes(source, node) == b"Exception",
        "scope_resolution" => {
            node.child_by_field_name("scope").is_none()
                && node
                    .child_by_field_name("name")
                    .is_some_and(|n| node_bytes(source, n) == b"Exception")
        }
        _ => false,
    }
}

fn prefer_name(config: &CopConfig) -> &'static str {
    if config.get_str("EnforcedStyle", "runtime_error") == "standard_error" {
        "StandardError"
    } else {
        "RuntimeError"
    }
}

fn report(
    cop: &InheritException,
    source: &SourceFile,
    at: Node<'_>,
    prefer: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, col) = source.offset_to_line_col(at.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!("Inherit from `{prefer}` instead of `Exception`."),
    ));
}

fn check_class(
    cop: &InheritException,
    source: &SourceFile,
    node: Node<'_>,
    prefer: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(sc) = node.child_by_field_name("superclass") else {
        return;
    };
    let mut cur = sc.walk();
    let Some(parent) = sc.named_children(&mut cur).next() else {
        return;
    };
    if exception_name(source, parent) {
        report(cop, source, parent, prefer, diagnostics);
    }
}

fn check_class_new(
    cop: &InheritException,
    source: &SourceFile,
    node: Node<'_>,
    prefer: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if call_method_name(source, node) != Some(b"new") {
        return;
    }
    let Some(recv) = call_receiver(node) else {
        return;
    };
    if node_bytes(source, recv) != b"Class" {
        return;
    }
    let Some(first) = argument_nodes(node).into_iter().next() else {
        return;
    };
    if exception_name(source, first) {
        report(cop, source, first, prefer, diagnostics);
    }
}

impl Cop for InheritException {
    fn name(&self) -> &'static str {
        "Lint/InheritException"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class", "call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let prefer = prefer_name(config);
        if node.kind() == "class" {
            check_class(self, source, node, prefer, diagnostics);
        } else {
            check_class_new(self, source, node, prefer, diagnostics);
        }
    }
}
