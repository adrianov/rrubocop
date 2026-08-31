//! RSpec/BeEql — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, method_node, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BeEql;

const LITERAL_KINDS: &[&str] = &[
    "true",
    "false",
    "nil",
    "integer",
    "float",
    "symbol",
    "simple_symbol",
];

fn eql_literal(source: &SourceFile, node: Node<'_>) -> bool {
    call_method_name(source, node) == Some(b"eql")
        && argument_nodes(node)
            .first()
            .is_some_and(|a| LITERAL_KINDS.contains(&a.kind()))
}

fn under_negated_expect(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(p) = node.parent() else {
        return false;
    };
    let host = if p.kind() == "argument_list" {
        p.parent()
    } else {
        Some(p)
    };
    matches!(
        host.and_then(|h| call_method_name(source, h)),
        Some(b"not_to") | Some(b"to_not")
    )
}

fn report_prefer_be(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let meth = method_node(node).unwrap_or(node);
    let (line, col) = source.offset_to_line_col(meth.start_byte());
    let mut diag = cop.diagnostic(source, line, col, "Prefer `be` over `eql`.".into());
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

impl Cop for BeEql {
    fn name(&self) -> &'static str {
        "RSpec/BeEql"
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
        &["call", "false", "integer", "nil", "symbol", "true", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if !eql_literal(source, node) || under_negated_expect(source, node) {
            return;
        }
        report_prefer_be(self, source, node, diagnostics, &mut corrections);
    }
}
