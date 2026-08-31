//! Layout/SpaceBeforeBrackets.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceBeforeBrackets;

fn lbracket_gap(bytes: &[u8], node: Node<'_>) -> Option<(usize, usize)> {
    let mut cur = node.walk();
    let lbrack = node.children(&mut cur).find(|c| c.kind() == "[")?;
    let recv_end = node.child(0).map(|c| c.end_byte()).unwrap_or(node.start_byte());
    if lbrack.start_byte() > recv_end && shared::has_hspace(bytes, recv_end, lbrack.start_byte()) {
        Some((recv_end, lbrack.start_byte()))
    } else {
        None
    }
}

impl Cop for SpaceBeforeBrackets {
    fn name(&self) -> &'static str { "Layout/SpaceBeforeBrackets" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["element_reference"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        let Some((recv_end, lb)) = lbracket_gap(source.as_bytes(), node) else { return; };
        report::report_fix(
            self, source, recv_end,
            "Remove the space before the opening brackets.".into(),
            diagnostics, &mut corrections, recv_end, lb, String::new(),
        );
    }
}
