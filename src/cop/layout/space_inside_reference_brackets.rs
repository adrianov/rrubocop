//! Layout/SpaceInsideReferenceBrackets.

use tree_sitter::Node;

use crate::cop::layout::space_delim::{self, DelimSpace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceInsideReferenceBrackets;

fn find_brackets<'a>(node: Node<'a>) -> Option<(Node<'a>, Node<'a>)> {
    let mut cur = node.walk();
    let kids: Vec<_> = node.children(&mut cur).collect();
    let lb = kids.iter().find(|c| c.kind() == "[").copied()?;
    let rb = kids.iter().rev().find(|c| c.kind() == "]").copied()?;
    Some((lb, rb))
}

fn check_empty(
    cop: &dyn Cop,
    source: &SourceFile,
    config: &CopConfig,
    lb: Node<'_>,
    d: &DelimSpace,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let want_e = config.get_str("EnforcedStyleForEmptyBrackets", "no_space") == "space";
    let has = d.inner_e > d.inner_s;
    if want_e == has {
        return;
    }
    let cmd = if want_e { "Use" } else { "Do not use" };
    space_delim::report_at(
        cop,
        source,
        lb.start_byte(),
        format!("{cmd} space inside empty reference brackets."),
        diagnostics,
        corrections,
        d.inner_s,
        d.inner_e,
        if want_e { " ".into() } else { String::new() },
    );
}

impl Cop for SpaceInsideReferenceBrackets {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideReferenceBrackets"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["element_reference"]
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
        let Some((lb, rb)) = find_brackets(node) else {
            return;
        };
        let Some(d) = space_delim::scan_inner(bytes, lb.end_byte(), rb.start_byte()) else {
            return;
        };
        if space_delim::is_blank_inner(bytes, d.inner_s, d.inner_e) {
            check_empty(self, source, config, lb, &d, diagnostics, &mut corrections);
            return;
        }
        let want = style == "space";
        let cmd = if want { "Use" } else { "Do not use" };
        let msg = format!("{cmd} space inside reference brackets.");
        space_delim::enforce_spaces(
            self,
            source,
            bytes,
            &d,
            want,
            lb.start_byte(),
            d.inner_e.saturating_sub(1),
            &msg,
            diagnostics,
            &mut corrections,
        );
    }
}
