//! Layout/EmptyComment.

use tree_sitter::Tree;

use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct EmptyComment;

fn is_empty_comment(text: &[u8]) -> bool {
    text.starts_with(b"#") && !text[1..].iter().any(|&b| b != b' ' && b != b'\t' && b != b'\r')
}

fn is_border(text: &[u8]) -> bool {
    text.len() >= 2 && text.iter().all(|&b| b == b'#')
}

fn line_start_of(bytes: &[u8], start: usize) -> usize {
    let mut ls = start;
    while ls > 0 && bytes[ls - 1] != b'\n' { ls -= 1; }
    ls
}

fn standalone_range(bytes: &[u8], line_start: usize, end: usize) -> (usize, usize) {
    let mut e = end;
    if e < bytes.len() && bytes[e] == b'\n' { e += 1; }
    (line_start, e)
}

fn inline_range(bytes: &[u8], line_start: usize, start: usize, end: usize) -> (usize, usize) {
    let mut s = start;
    while s > line_start && matches!(bytes[s - 1], b' ' | b'\t') { s -= 1; }
    (s, end)
}

fn removal_range(bytes: &[u8], start: usize, end: usize) -> (usize, usize) {
    let line_start = line_start_of(bytes, start);
    let before = &bytes[line_start..start];
    if before.iter().all(|&b| b == b' ' || b == b'\t') {
        standalone_range(bytes, line_start, end)
    } else {
        inline_range(bytes, line_start, start, end)
    }
}

fn report_empty(
    cop: &dyn Cop, source: &SourceFile, bytes: &[u8], start: usize, end: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(start);
    let mut diag = cop.diagnostic(source, line, col, "Source code comment is empty.".into());
    if let Some(corr) = corrections {
        let (s, e) = removal_range(bytes, start, end);
        corr.push(Correction {
            start: s, end: e, replacement: String::new(),
            cop_name: cop.name(), cop_index: 0,
        });
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

impl Cop for EmptyComment {
    fn name(&self) -> &'static str { "Layout/EmptyComment" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, tree: &Tree, code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = code_map;
        let allow_border = config.get_bool("AllowBorderComment", true);
        let bytes = source.as_bytes();
        for comment in shared::collect_comments(tree.root_node()) {
            let start = comment.start_byte();
            let end = comment.end_byte();
            let text = &bytes[start..end];
            if is_empty_comment(text) || (!allow_border && is_border(text)) {
                report_empty(self, source, bytes, start, end, diagnostics, &mut corrections);
            }
        }
    }
}
