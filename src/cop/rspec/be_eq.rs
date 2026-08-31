//! RSpec/BeEq — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, method_node, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BeEq;

fn eq_bool_or_nil(source: &SourceFile, node: Node<'_>) -> bool {
    call_method_name(source, node) == Some(b"eq")
        && argument_nodes(node)
            .first()
            .is_some_and(|a| matches!(a.kind(), "true" | "false" | "nil"))
}

fn report_prefer_be(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    msg: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let meth = method_node(node).unwrap_or(node);
    let (line, col) = source.offset_to_line_col(meth.start_byte());
    let mut diag = cop.diagnostic(source, line, col, msg.into());
    if push_replace(
        corrections,
        meth.start_byte(),
        meth.end_byte(),
        "be",
        cop.name(),
    ) {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

impl Cop for BeEq {
    fn name(&self) -> &'static str {
        "RSpec/BeEq"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn safe_autocorrect(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "false", "nil", "true", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if !eq_bool_or_nil(source, node) {
            return;
        }
        report_prefer_be(
            self,
            source,
            node,
            "Prefer `be` over `eq`.",
            diagnostics,
            &mut corrections,
        );
    }
}
