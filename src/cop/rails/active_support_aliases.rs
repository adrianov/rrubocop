//! Rails/ActiveSupportAliases — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, method_node, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ActiveSupportAliases;

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
        let prefer = match method {
            b"starts_with?" => "start_with?",
            b"ends_with?" => "end_with?",
            _ => return,
        };
        let current = std::str::from_utf8(method).unwrap_or("");
        let meth = method_node(node).unwrap_or(node);
        let (line, col) = source.offset_to_line_col(meth.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            format!("Use `{prefer}` instead of `{current}`."),
        );
        if push_replace(
            &mut corrections,
            meth.start_byte(),
            meth.end_byte(),
            prefer,
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}
