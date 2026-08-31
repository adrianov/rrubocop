//! RSpec/ClassCheck — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, method_node, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ClassCheck;

fn style_prefer(style: &str, method: &[u8]) -> Option<&'static str> {
    let is_candidate = matches!(
        method,
        b"be_a" | b"be_an" | b"be_kind_of" | b"be_a_kind_of"
    );
    if !is_candidate {
        return None;
    }
    let (ok, prefer) = if style == "be_a" {
        (matches!(method, b"be_a" | b"be_an"), "be_a")
    } else {
        (matches!(method, b"be_kind_of" | b"be_a_kind_of"), "be_kind_of")
    };
    (!ok).then_some(prefer)
}

fn report_prefer(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    prefer: &str,
    method: &[u8],
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
        format!("Prefer `{prefer}` over `{current}`."),
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

impl Cop for ClassCheck {
    fn name(&self) -> &'static str {
        "RSpec/ClassCheck"
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
        let style = config.get_str("EnforcedStyle", "be_a");
        let Some(prefer) = style_prefer(style, method) else {
            return;
        };
        report_prefer(
            self,
            source,
            node,
            prefer,
            method,
            diagnostics,
            &mut corrections,
        );
    }
}
