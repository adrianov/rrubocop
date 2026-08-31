//! Layout/SpaceInLambdaLiteral.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceInLambdaLiteral;

fn arrow_paren_gap(bytes: &[u8], node: Node<'_>) -> Option<(usize, usize, bool)> {
    if !bytes[node.start_byte()..].starts_with(b"->") { return None; }
    let arrow_end = node.start_byte() + 2;
    let rest = &bytes[arrow_end..node.end_byte()];
    let paren_rel = rest.iter().position(|&b| b == b'(')?;
    let between = &rest[..paren_rel];
    if between.iter().any(|&b| b == b'\n') { return None; }
    let has_space = between.iter().any(|&b| b == b' ' || b == b'\t');
    Some((arrow_end, paren_rel, has_space))
}

impl Cop for SpaceInLambdaLiteral {
    fn name(&self) -> &'static str { "Layout/SpaceInLambdaLiteral" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["lambda"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let want = config.get_str("EnforcedStyle", "require_space") != "require_no_space";
        let Some((arrow_end, paren_rel, has_space)) = arrow_paren_gap(source.as_bytes(), node) else {
            return;
        };
        if want == has_space { return; }
        let msg = if want {
            "Use a space between `->` and `(` in lambda literals."
        } else {
            "Do not use spaces between `->` and `(` in lambda literals."
        };
        report::report_fix(
            self, source, arrow_end, msg.into(), diagnostics, &mut corrections,
            arrow_end, arrow_end + paren_rel,
            if want { " ".into() } else { String::new() },
        );
    }
}
