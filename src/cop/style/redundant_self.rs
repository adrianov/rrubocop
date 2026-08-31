//! Style/RedundantSelf — avoid unnecessary self.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RedundantSelf;

impl Cop for RedundantSelf {
    fn name(&self) -> &'static str {
        "Style/RedundantSelf"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(recv) = call_receiver(node) else {
            return;
        };
        if recv.kind() != "self" || !self_is_redundant(source, node) {
            return;
        }
        report(self, source, node, recv, diagnostics, &mut corrections);
    }
}

fn report(
    cop: &RedundantSelf,
    source: &SourceFile,
    node: Node<'_>,
    recv: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(recv.start_byte());
    let mut diag = cop.diagnostic(source, line, col, "Redundant `self` detected.".to_string());
    if let Some(corr) = corrections.as_mut() {
        let meth = node.child_by_field_name("method").unwrap_or(node);
        corr.push(Correction {
            start: recv.start_byte(),
            end: meth.start_byte(),
            replacement: String::new(),
            cop_name: cop.name(),
            cop_index: 0,
        });
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn self_is_redundant(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(method) = call_method_name(source, node) else {
        return false;
    };
    if method == b"class" || method.starts_with(b"[") {
        return false;
    }
    method
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'_' | b'!' | b'?' | b'='))
}
