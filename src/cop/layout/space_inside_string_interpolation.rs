//! Layout/SpaceInsideStringInterpolation.

use tree_sitter::Node;

use crate::cop::layout::space_delim;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceInsideStringInterpolation;

impl Cop for SpaceInsideStringInterpolation {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideStringInterpolation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["interpolation"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "no_space");
        let bytes = source.as_bytes();
        if !bytes[node.start_byte()..].starts_with(b"#{") {
            return;
        }
        let inner_s = node.start_byte() + 2;
        let inner_e = node.end_byte().saturating_sub(1);
        let Some(d) = space_delim::scan_inner(bytes, inner_s, inner_e) else {
            return;
        };
        let want = style == "space";
        let cmd = if want { "Use" } else { "Do not use" };
        let msg = format!("{cmd} space inside string interpolation.");
        space_delim::enforce_spaces(
            self,
            source,
            bytes,
            &d,
            want,
            node.start_byte(),
            d.inner_e,
            &msg,
            diagnostics,
            &mut corrections,
        );
    }
}
