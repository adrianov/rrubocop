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
        let bytes = source.as_bytes();
        let text = &bytes[node.start_byte()..node.end_byte()];
        if !is_array_percent(text) {
            return;
        }
        let Some((open_idx, crel)) = percent_span(text) else {
            return;
        };
        let inner = &text[open_idx + 1..crel];
        let Some(rel) = double_space_at(inner) else {
            return;
        };
        let abs = node.start_byte() + open_idx + 1 + rel;
        report_double(
            self,
            source,
            bytes,
            abs,
            node.start_byte() + crel,
            diagnostics,
            &mut corrections,
        );
    }
}
