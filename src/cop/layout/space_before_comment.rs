//! Layout/SpaceBeforeComment.

use tree_sitter::Tree;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct SpaceBeforeComment;

fn line_prefix<'a>(bytes: &'a [u8], start: usize) -> &'a [u8] {
    let line_start = bytes[..start]
        .iter()
        .rposition(|&b| b == b'\n')
        .map_or(0, |i| i + 1);
    let before = &bytes[line_start..start];
    before.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(before)
}

fn only_indent(bytes: &[u8], start: usize) -> bool {
    line_prefix(bytes, start)
        .iter()
        .all(|&b| b == b' ' || b == b'\t')
}

fn is_char_esc(bytes: &[u8], start: usize) -> bool {
    start >= 3 && bytes[start - 2] == b'\\' && bytes[start - 3] == b'?'
}

fn needs_space(bytes: &[u8], start: usize) -> bool {
    if start == 0 {
        return false;
    }
    let prev = bytes[start - 1];
    if prev == b'\n' || prev == b'\r' || only_indent(bytes, start) {
        return false;
    }
    (prev != b' ' && prev != b'\t') || (prev == b' ' && is_char_esc(bytes, start))
}

fn check_one(
    cop: &dyn Cop,
    source: &SourceFile,
    bytes: &[u8],
    start: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !needs_space(bytes, start) {
        return;
    }
    report::insert_space(
        cop,
        source,
        start,
        "Put a space before an end-of-line comment.".into(),
        diagnostics,
        corrections,
        start,
    );
}

impl Cop for SpaceBeforeComment {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeComment"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = (code_map, config);
        let bytes = source.as_bytes();
        for comment in shared::collect_comments(tree.root_node()) {
            check_one(
                self,
                source,
                bytes,
                comment.start_byte(),
                diagnostics,
                &mut corrections,
            );
        }
    }
}
