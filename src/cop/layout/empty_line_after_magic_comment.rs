//! Layout/EmptyLineAfterMagicComment.

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLineAfterMagicComment;

fn trim_ws(line: &[u8]) -> &[u8] {
    let s = line.iter().position(|&b| b != b' ' && b != b'\t' && b != b'\r').unwrap_or(line.len());
    &line[s..]
}

fn is_magic(line: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(line) else {
        return false;
    };
    let t = s.trim();
    if !t.starts_with('#') {
        return false;
    }
    let after = t[1..].trim_start();
    if after.starts_with('#') {
        return false;
    }
    let lower = after.to_ascii_lowercase();
    lower.starts_with("frozen_string_literal:")
        || lower.starts_with("frozen-string-literal:")
        || lower.starts_with("shareable_constant_value:")
        || lower.starts_with("typed:")
        || lower.starts_with("rbs_inline:")
        || {
            let s = lower.trim_start_matches(|c: char| c == '-' || c == '*' || c == ' ');
            s.starts_with("encoding:")
                || s.starts_with("coding:")
                || s.starts_with("encoding =")
                || (s.contains("-*-") && (s.contains("encoding:") || s.contains("coding:")))
        }
}

fn line_at<'a>(lines: &[&'a [u8]], idx: usize) -> &'a [u8] {
    let line = lines[idx];
    if idx == 0 { line.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(line) } else { line }
}

fn content_limit(lines: &[&[u8]]) -> Option<usize> {
    for (idx, _) in lines.iter().enumerate() {
        let line = line_at(lines, idx);
        let trimmed = trim_ws(line);
        if trimmed.is_empty() || trimmed.starts_with(b"#") { continue; }
        if line.strip_suffix(b"\r").unwrap_or(line) == b"__END__" { return None; }
        return Some(idx);
    }
    Some(lines.len())
}

fn last_magic_line(lines: &[&[u8]]) -> Option<usize> {
    let limit = content_limit(lines)?;
    let mut last = None;
    for idx in 0..limit {
        if is_magic(line_at(lines, idx)) { last = Some(idx); }
    }
    last
}

impl Cop for EmptyLineAfterMagicComment {
    fn name(&self) -> &'static str { "Layout/EmptyLineAfterMagicComment" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_lines(
        &self, source: &SourceFile, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        let lines: Vec<&[u8]> = source.lines().collect();
        let Some(last_magic) = last_magic_line(&lines) else { return; };
        let next = last_magic + 1;
        if next >= lines.len() { return; }
        let blank = lines[next].iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r');
        if blank { return; }
        report::insert_newline(
            self, source, next + 1,
            "Add an empty line after magic comments.".into(),
            diagnostics, &mut corrections,
        );
    }
}
