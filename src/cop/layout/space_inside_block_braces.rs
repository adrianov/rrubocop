//! Layout/SpaceInsideBlockBraces.

use tree_sitter::Node;

use crate::cop::layout::space_delim::{self, DelimSpace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceInsideBlockBraces;

fn find_braces<'a>(node: Node<'a>) -> Option<(Node<'a>, Node<'a>)> {
    let mut cur = node.walk();
    let kids: Vec<_> = node.children(&mut cur).collect();
    let lbrace = kids.iter().find(|c| c.kind() == "{").copied()?;
    let rbrace = kids.iter().rev().find(|c| c.kind() == "}").copied()?;
    Some((lbrace, rbrace))
}

fn check_empty(
    cop: &dyn Cop,
    source: &SourceFile,
    config: &CopConfig,
    lbrace: Node<'_>,
    d: &DelimSpace,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let want_e = config.get_str("EnforcedStyleForEmptyBraces", "no_space") == "space";
    let has = d.inner_e > d.inner_s;
    if want_e == has {
        return;
    }
    let msg = if want_e {
        "Space missing inside empty braces."
    } else {
        "Space detected inside empty braces."
    };
    space_delim::report_at(
        cop,
        source,
        lbrace.start_byte(),
        msg.into(),
        diagnostics,
        corrections,
        d.inner_s,
        d.inner_e,
        if want_e { " ".into() } else { String::new() },
        None,
    );
}

fn check_nonempty(
    cop: &dyn Cop,
    source: &SourceFile,
    bytes: &[u8],
    d: &DelimSpace,
    want: bool,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if want {
        space_delim::add_space_after(
            cop, source, d, d.inner_s, "Space missing inside {.".into(), diagnostics, corrections,
        );
        space_delim::add_space_before(
            cop, source, d, d.inner_e, "Space missing inside }.".into(), diagnostics, corrections,
        );
    } else {
        let mut correctable = true;
        space_delim::strip_space_after(
            cop, source, bytes, d, "Space detected inside {.".into(), diagnostics, corrections,
            &mut correctable,
        );
        space_delim::strip_space_before(
            cop, source, bytes, d, "Space detected inside }.".into(), diagnostics, corrections,
            &mut correctable,
        );
    }
}

impl Cop for SpaceInsideBlockBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideBlockBraces"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["block"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "space");
        let bytes = source.as_bytes();
        let Some((lbrace, rbrace)) = find_braces(node) else {
            return;
        };
        let Some(d) = space_delim::scan_inner(bytes, lbrace.end_byte(), rbrace.start_byte()) else {
            return;
        };
        if d.inner_e <= d.inner_s {
            return;
        }
        let empty = bytes[d.inner_s..d.inner_e]
            .iter()
            .all(|&b| b == b' ' || b == b'\t');
        if empty {
            check_empty(self, source, config, lbrace, &d, diagnostics, &mut corrections);
            return;
        }
        check_nonempty(
            self,
            source,
            bytes,
            &d,
            style != "no_space",
            diagnostics,
            &mut corrections,
        );
    }
}
