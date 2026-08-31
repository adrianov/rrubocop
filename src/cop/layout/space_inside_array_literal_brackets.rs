//! Layout/SpaceInsideArrayLiteralBrackets.

use tree_sitter::Node;

use crate::cop::layout::space_delim::{self, DelimSpace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceInsideArrayLiteralBrackets;

fn check_empty(
    cop: &dyn Cop, source: &SourceFile, config: &CopConfig, d: &DelimSpace,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let want_empty = config.get_str("EnforcedStyleForEmptyBrackets", "no_space") == "space";
    let has = d.sp_a || d.sp_b || (d.inner_e > d.inner_s);
    if want_empty && !has {
        space_delim::report_at(
            cop, source, d.inner_s.saturating_sub(1),
            "Use space inside empty array brackets.".into(),
            diagnostics, corrections, d.inner_s, d.inner_e, " ".into(),
        );
    } else if !want_empty && d.inner_e > d.inner_s {
        space_delim::report_at(
            cop, source, d.inner_s,
            "Do not use space inside empty array brackets.".into(),
            diagnostics, corrections, d.inner_s, d.inner_e, String::new(),
        );
    }
}

fn check_filled(
    cop: &dyn Cop, source: &SourceFile, bytes: &[u8], node: Node<'_>, d: &DelimSpace, style: &str,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let want = style == "space" || style == "compact";
    let cmd = if want { "Use" } else { "Do not use" };
    let msg = format!("{cmd} space inside array brackets.");
    space_delim::enforce_spaces(
        cop, source, bytes, d, want, node.start_byte(), d.inner_e, &msg,
        diagnostics, corrections,
    );
}

impl Cop for SpaceInsideArrayLiteralBrackets {
    fn name(&self) -> &'static str { "Layout/SpaceInsideArrayLiteralBrackets" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["array"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "no_space");
        let bytes = source.as_bytes();
        if bytes.get(node.start_byte()) != Some(&b'[') { return; }
        let Some(d) = space_delim::scan_inner(bytes, node.start_byte() + 1, node.end_byte() - 1) else { return; };
        if space_delim::is_blank_inner(bytes, d.inner_s, d.inner_e) {
            check_empty(self, source, config, &d, diagnostics, &mut corrections);
        } else {
            check_filled(self, source, bytes, node, &d, style, diagnostics, &mut corrections);
        }
    }
}
