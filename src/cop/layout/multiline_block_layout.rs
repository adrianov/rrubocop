//! Layout/MultilineBlockLayout.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineBlockLayout;

fn block_end_line(source: &SourceFile, node: Node<'_>) -> usize {
    shared::end_keyword(node)
        .or_else(|| shared::last_child_kind(node, "}"))
        .map(|e| shared::node_line(source, e))
        .unwrap_or_else(|| shared::node_line(source, node))
}

fn check_params(cop: &dyn Cop, source: &SourceFile, node: Node<'_>, start_line: usize, diagnostics: &mut Vec<Diagnostic>) {
    let Some(params) = shared::child_kind(node, "block_parameters") else { return; };
    if shared::node_line(source, params) == start_line { return; }
    let (l, c) = source.offset_to_line_col(params.start_byte());
    diagnostics.push(cop.diagnostic(
        source, l, c,
        "Block argument expression is not on the same line as the block start.".into(),
    ));
}

fn first_body<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .find(|n| !matches!(n.kind(), "block_parameters" | "comment"))
}

fn check_body(
    cop: &dyn Cop, source: &SourceFile, node: Node<'_>, start_line: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(first) = first_body(node) else { return; };
    if shared::node_line(source, first) != start_line { return; }
    report::report_fix(
        cop, source, first.start_byte(),
        "Block body expression is on the same line as the block start.".into(),
        diagnostics, corrections,
        first.start_byte(), first.start_byte(), "\n".into(),
    );
}

impl Cop for MultilineBlockLayout {
    fn name(&self) -> &'static str { "Layout/MultilineBlockLayout" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["do_block", "block"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        let start_line = shared::node_line(source, node);
        if start_line == block_end_line(source, node) { return; }
        check_params(self, source, node, start_line, diagnostics);
        check_body(self, source, node, start_line, diagnostics, &mut corrections);
    }
}
