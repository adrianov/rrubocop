//! Layout/SpaceAfterColon — ported from RuboCop/nitrocop (tree-sitter).

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceAfterColon;

fn report_colon(
    cop: &dyn Cop,
    source: &SourceFile,
    colon: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let after = colon + 1;
    if matches!(source.as_bytes().get(after), Some(b) if b.is_ascii_whitespace()) {
        return;
    }
    report::insert_space(
        cop,
        source,
        colon,
        "Space missing after colon.".into(),
        diagnostics,
        corrections,
        after,
    );
}

fn check_kw_param(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(name) = node.child_by_field_name("name") else {
        return;
    };
    let end = name.end_byte();
    if end > 0 && source.as_bytes().get(end - 1) == Some(&b':') {
        report_colon(cop, source, end - 1, diagnostics, corrections);
    }
}

fn colon_in_pair(bytes: &[u8], key: Node<'_>, value: Node<'_>) -> Option<usize> {
    let from = key.end_byte();
    let to = value.start_byte();
    gap_colon(bytes, from, to).or_else(|| key_trailing_colon(bytes, key.end_byte()))
}

fn gap_colon(bytes: &[u8], from: usize, to: usize) -> Option<usize> {
    (to > from)
        .then(|| bytes[from..to].iter().rposition(|&b| b == b':').map(|r| from + r))
        .flatten()
}

fn key_trailing_colon(bytes: &[u8], end: usize) -> Option<usize> {
    (bytes.get(end.saturating_sub(1)) == Some(&b':')).then_some(end - 1)
}

fn check_pair(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(key) = node.child_by_field_name("key") else {
        return;
    };
    let Some(value) = node.child_by_field_name("value") else {
        return;
    };
    let Some(colon) = colon_in_pair(source.as_bytes(), key, value) else {
        return;
    };
    report_colon(cop, source, colon, diagnostics, corrections);
}

impl Cop for SpaceAfterColon {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterColon"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["pair", "keyword_parameter"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if node.kind() == "keyword_parameter" {
            check_kw_param(self, source, node, diagnostics, &mut corrections);
        } else {
            check_pair(self, source, node, diagnostics, &mut corrections);
        }
    }
}
