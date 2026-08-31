//! Layout/SpaceBeforeBlockBraces.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceBeforeBlockBraces;

fn find_lbrace<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.walk();
    node.children(&mut cur).find(|c| c.kind() == "{")
}

fn ws_before(bytes: &[u8], start: usize) -> usize {
    let mut ws = start;
    while ws > 0 && matches!(bytes[ws - 1], b' ' | b'\t') { ws -= 1; }
    ws
}

impl Cop for SpaceBeforeBlockBraces {
    fn name(&self) -> &'static str { "Layout/SpaceBeforeBlockBraces" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["block"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let want = config.get_str("EnforcedStyle", "space") != "no_space";
        let bytes = source.as_bytes();
        let Some(lbrace) = find_lbrace(node) else { return; };
        let start = lbrace.start_byte();
        if start == 0 { return; }
        let ws_start = ws_before(bytes, start);
        let has_space = ws_start < start;
        if want && !has_space {
            report::insert_space(
                self, source, start, "Space missing to the left of {.".into(),
                diagnostics, &mut corrections, start,
            );
        } else if !want && has_space {
            report::report_fix(
                self, source, ws_start, "Space detected to the left of {.".into(),
                diagnostics, &mut corrections, ws_start, start, String::new(),
            );
        }
    }
}
