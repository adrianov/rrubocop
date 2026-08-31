//! RSpec/NotToNot — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, method_node, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct NotToNot;

impl Cop for NotToNot {
    fn name(&self) -> &'static str {
        "RSpec/NotToNot"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        let prefer = config.get_str("EnforcedStyle", "not_to");
        let wrong = if prefer == "not_to" { "to_not" } else { "not_to" };
        if method != wrong.as_bytes() {
            return;
        }
        let meth = method_node(node).unwrap_or(node);
        let (line, col) = source.offset_to_line_col(meth.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            format!("Prefer `{prefer}` over `{wrong}`."),
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
