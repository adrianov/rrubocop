//! Layout/SpaceInsideHashLiteralBraces.

use tree_sitter::Node;

use crate::cop::layout::space_delim;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceInsideHashLiteralBraces;

fn side_msg(want: bool, brace: &str) -> String {
    let problem = if want { "missing" } else { "detected" };
    format!("Space inside {brace} {problem}.")
}

fn check_open(
    cop: &dyn Cop,
    source: &SourceFile,
    bytes: &[u8],
    d: &space_delim::DelimSpace,
    open_off: usize,
    want: bool,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let msg = side_msg(want, "{");
    if want {
        space_delim::add_space_after(cop, source, d, open_off, msg, diagnostics, corrections);
    } else {
        space_delim::strip_space_after(cop, source, bytes, d, msg, diagnostics, corrections);
    }
}

fn check_close(
    cop: &dyn Cop,
    source: &SourceFile,
    bytes: &[u8],
    d: &space_delim::DelimSpace,
    want: bool,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let msg = side_msg(want, "}");
    if want {
        space_delim::add_space_before(cop, source, d, d.inner_e, msg, diagnostics, corrections);
    } else {
        space_delim::strip_space_before(cop, source, bytes, d, msg, diagnostics, corrections);
    }
}

impl Cop for SpaceInsideHashLiteralBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideHashLiteralBraces"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        // RuboCop aliases `on_hash_pattern` → `on_hash`.
        &["hash", "hash_pattern"]
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
        if bytes.get(node.start_byte()) != Some(&b'{') {
            return;
        }
        let inner_s = node.start_byte() + 1;
        let inner_e = node.end_byte() - 1;
        if inner_e <= inner_s {
            return;
        }
        let Some(d) = space_delim::scan_inner(bytes, inner_s, inner_e) else {
            return;
        };
        let want = style != "no_space";
        check_open(
            self, source, bytes, &d, node.start_byte(), want, diagnostics, &mut corrections,
        );
        check_close(self, source, bytes, &d, want, diagnostics, &mut corrections);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(SpaceInsideHashLiteralBraces, "cops/layout/space_inside_hash_literal_braces");
}
