//! Rails/ActiveSupportAliases — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, method_node, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ActiveSupportAliases;

fn prefer_alias(method: &[u8]) -> Option<&'static str> {
    match method {
        b"starts_with?" => Some("start_with?"),
        b"ends_with?" => Some("end_with?"),
        _ => None,
    }
}

fn report_alias(
    cop: &ActiveSupportAliases,
    source: &SourceFile,
    node: Node<'_>,
    method: &[u8],
    prefer: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let current = std::str::from_utf8(method).unwrap_or("");
    let meth = method_node(node).unwrap_or(node);
    let (line, col) = source.offset_to_line_col(meth.start_byte());
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        format!("Use `{prefer}` instead of `{current}`."),
    );
    if push_replace(
        corrections,
        meth.start_byte(),
        meth.end_byte(),
        prefer,
        cop.name(),
    ) {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

impl Cop for ActiveSupportAliases {
    fn name(&self) -> &'static str {
        "Rails/ActiveSupportAliases"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["array", "call", "string", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        let Some(prefer) = prefer_alias(method) else {
            return;
        };
        // RuboCop-Rails: only string receivers for starts_with?/ends_with?.
        if !call_receiver(node).is_some_and(|r| r.kind() == "string") {
            return;
        }
        report_alias(self, source, node, method, prefer, diagnostics, &mut corrections);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ActiveSupportAliases, "cops/rails/active_support_aliases");
}
