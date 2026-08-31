//! Style/CommentAnnotation — breadth-first tree-sitter port.

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CommentAnnotation;

impl Cop for CommentAnnotation {
    fn name(&self) -> &'static str {
        "Style/CommentAnnotation"
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        for (idx, line) in source.lines().enumerate() {
            check_line(self, source, idx, line, diagnostics);
        }
    }
}

fn check_line(
    cop: &CommentAnnotation,
    source: &SourceFile,
    idx: usize,
    line: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let s = String::from_utf8_lossy(line);
    let trimmed = s.trim_start();
    if !trimmed.starts_with('#') {
        return;
    }
    let body = trimmed.trim_start_matches('#').trim_start();
    for kw in ["TODO", "FIXME", "OPTIMIZE", "HACK", "REVIEW", "NOTE"] {
        if let Some(msg) = annotation_msg(body, kw) {
            let (line_n, col) =
                source.offset_to_line_col(source.line_start(idx + 1).unwrap_or(0));
            diagnostics.push(cop.diagnostic(source, line_n, col, msg));
            return;
        }
    }
}

fn annotation_msg(body: &str, kw: &str) -> Option<String> {
    if !body.starts_with(kw) {
        return None;
    }
    let rest = &body[kw.len()..];
    if rest.is_empty() || rest.starts_with(':') || rest.starts_with(' ') {
        if !rest.starts_with(':') {
            return Some(format!(
                "Annotation keywords like `{kw}` should be followed by a colon."
            ));
        }
    }
    None
}
