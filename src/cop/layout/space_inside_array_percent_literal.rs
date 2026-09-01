//! Layout/SpaceInsideArrayPercentLiteral.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceInsideArrayPercentLiteral;

fn is_array_percent(text: &[u8]) -> bool {
    text.starts_with(b"%w")
        || text.starts_with(b"%W")
        || text.starts_with(b"%i")
        || text.starts_with(b"%I")
}

fn matching_close(open: u8) -> u8 {
    match open {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        b'<' => b'>',
        c => c,
    }
}

fn percent_span(text: &[u8]) -> Option<(usize, usize)> {
    let open_idx = 2 + text[2..].iter().position(|&b| !b.is_ascii_alphanumeric()).unwrap_or(0);
    if open_idx >= text.len() {
        return None;
    }
    let close = matching_close(text[open_idx]);
    let crel = text.iter().rposition(|&b| b == close)?;
    Some((open_idx, crel))
}

fn double_space_at(inner: &[u8]) -> Option<usize> {
    inner.windows(2).position(|w| w == b"  ")
}

fn collapse_spaces(bytes: &[u8], start: usize, limit: usize) -> usize {
    let mut e = start;
    while e < limit && bytes[e] == b' ' {
        e += 1;
    }
    e
}

fn report_double(
    cop: &dyn Cop,
    source: &SourceFile,
    bytes: &[u8],
    abs: usize,
    limit: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let end = collapse_spaces(bytes, abs, limit);
    report::report_fix(
        cop,
        source,
        abs,
        "Use only a single space inside array percent literal.".into(),
        diagnostics,
        corrections,
        abs,
        end,
        " ".into(),
    );
}

fn singleline_percent_inner(text: &[u8]) -> Option<(usize, usize, &[u8])> {
    let (open_idx, crel) = percent_span(text)?;
    let inner = &text[open_idx + 1..crel];
    if inner.contains(&b'\n') {
        return None;
    }
    Some((open_idx, crel, inner))
}

fn double_space_abs(source: &SourceFile, node: Node<'_>) -> Option<(usize, usize)> {
    let text = &source.as_bytes()[node.start_byte()..node.end_byte()];
    if !is_array_percent(text) {
        return None;
    }
    let (open_idx, crel, inner) = singleline_percent_inner(text)?;
    let rel = double_space_at(inner)?;
    Some((node.start_byte() + open_idx + 1 + rel, node.start_byte() + crel))
}

impl Cop for SpaceInsideArrayPercentLiteral {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideArrayPercentLiteral"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["string_array", "symbol_array", "%w", "%W", "%i", "%I"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        let Some((abs, limit)) = double_space_abs(source, node) else {
            return;
        };
        report_double(
            self,
            source,
            source.as_bytes(),
            abs,
            limit,
            diagnostics,
            &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(SpaceInsideArrayPercentLiteral, "cops/layout/space_inside_array_percent_literal");

    #[test]
    fn no_offense_multiline_fixture() {
        crate::testutil::assert_cop_no_offenses_full(
            &SpaceInsideArrayPercentLiteral,
            include_bytes!("../../../tests/fixtures/cops/layout/space_inside_array_percent_literal/no_offense_multiline.rb"),
        );
    }
}
