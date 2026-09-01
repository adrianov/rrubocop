//! Layout/SpaceInsideParens.

use tree_sitter::Node;

use crate::cop::layout::space_delim::{self, DelimSpace};
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceInsideParens;

fn paren_inner(bytes: &[u8], node: Node<'_>) -> Option<(usize, usize)> {
    if bytes.get(node.start_byte()) != Some(&b'(') {
        return None;
    }
    if bytes.get(node.end_byte().saturating_sub(1)) != Some(&b')') {
        return None;
    }
    let inner_s = node.start_byte() + 1;
    let inner_e = node.end_byte() - 1;
    (inner_e > inner_s).then_some((inner_s, inner_e))
}

fn hanging_close(source: &SourceFile, node: Node<'_>) -> bool {
    let close_off = node.end_byte() - 1;
    let (_, close_col) = source.offset_to_line_col(close_off);
    shared::line_indent(source, close_off) == close_col
}

fn comment_after_open(bytes: &[u8], inner_s: usize, inner_e: usize) -> bool {
    let mut i = inner_s;
    while i < inner_e && matches!(bytes[i], b' ' | b'\t') {
        i += 1;
    }
    bytes.get(i) == Some(&b'#')
}

fn enforce_paren_spaces(
    cop: &SpaceInsideParens,
    source: &SourceFile,
    bytes: &[u8],
    node: Node<'_>,
    d: &DelimSpace,
    want: bool,
    hanging: bool,
    skip_open_space: bool,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let msg = if want {
        "No space inside parentheses detected."
    } else {
        "Space inside parentheses detected."
    };
    let close_off = node.end_byte() - 1;
    let mut correctable = true;
    if !skip_open_space {
        if want {
            space_delim::add_space_after(
                cop,
                source,
                d,
                node.start_byte(),
                msg.into(),
                diagnostics,
                corrections,
            );
        } else {
            space_delim::strip_space_after(
                cop, source, bytes, d, msg.into(), diagnostics, corrections, &mut correctable,
            );
        }
    }
    if hanging {
        return;
    }
    if want {
        space_delim::add_space_before(
            cop, source, d, close_off, msg.into(), diagnostics, corrections,
        );
    } else {
        space_delim::strip_space_before(
            cop, source, bytes, d, msg.into(), diagnostics, corrections, &mut correctable,
        );
    }
}

impl Cop for SpaceInsideParens {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideParens"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &[
            "parenthesized_statements",
            "argument_list",
            "method_parameters",
            "block_parameters",
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let bytes = source.as_bytes();
        let Some((inner_s, inner_e)) = paren_inner(bytes, node) else {
            return;
        };
        let Some(d) = space_delim::scan_inner(bytes, inner_s, inner_e) else {
            return;
        };
        enforce_paren_spaces(
            self,
            source,
            bytes,
            node,
            &d,
            config.get_str("EnforcedStyle", "no_space") != "no_space",
            hanging_close(source, node),
            comment_after_open(bytes, inner_s, inner_e),
            diagnostics,
            &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(SpaceInsideParens, "cops/layout/space_inside_parens");
}
