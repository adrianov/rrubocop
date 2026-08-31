//! Layout/SpaceInsideParens.

use tree_sitter::Node;

use crate::cop::layout::space_delim;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceInsideParens;

fn paren_inner(bytes: &[u8], node: Node<'_>) -> Option<(usize, usize)> {
    if bytes.get(node.start_byte()) != Some(&b'(') { return None; }
    if bytes.get(node.end_byte().saturating_sub(1)) != Some(&b')') { return None; }
    let inner_s = node.start_byte() + 1;
    let inner_e = node.end_byte() - 1;
    if inner_e <= inner_s { None } else { Some((inner_s, inner_e)) }
}

impl Cop for SpaceInsideParens {
    fn name(&self) -> &'static str { "Layout/SpaceInsideParens" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["parenthesized_statements", "argument_list", "method_parameters", "block_parameters"]
    }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "space");
        let bytes = source.as_bytes();
        let Some((inner_s, inner_e)) = paren_inner(bytes, node) else { return; };
        let Some(d) = space_delim::scan_inner(bytes, inner_s, inner_e) else { return; };
        let want = style != "no_space";
        let msg = if want { "No space inside parentheses detected." } else { "Space inside parentheses detected." };
        space_delim::enforce_spaces(
            self, source, bytes, &d, want, node.start_byte(), d.inner_e, msg,
            diagnostics, &mut corrections,
        );
    }
}
