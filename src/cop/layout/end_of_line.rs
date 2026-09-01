//! Layout/EndOfLine — simplified port from nitrocop (line-based).

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EndOfLine;

fn check_lf(
    cop: &dyn Cop, source: &SourceFile,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut byte_offset = 0usize;
    for line in source.lines() {
        if line.ends_with(b"\r") {
            let cr_offset = byte_offset + line.len() - 1;
            report::report_fix(
                cop, source, cr_offset, "Carriage return character detected.".into(),
                diagnostics, corrections, cr_offset, cr_offset + 1, String::new(),
            );
            break;
        }
        byte_offset += line.len() + 1;
    }
}

fn skip_last_crlf_line(source: &SourceFile, lines: &[&[u8]], i: usize) -> bool {
    if i != lines.len() - 1 { return false; }
    lines[i].is_empty() || !source.as_bytes().ends_with(b"\n")
}

fn check_one_crlf(
    cop: &dyn Cop, source: &SourceFile, line: &[u8], byte_offset: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) -> bool {
    if line.ends_with(b"\r") { return false; }
    let newline_offset = byte_offset + line.len();
    report::report_fix(
        cop, source, newline_offset, "Carriage return character missing.".into(),
        diagnostics, corrections, newline_offset, newline_offset, "\r".into(),
    );
    true
}

fn check_crlf(
    cop: &dyn Cop, source: &SourceFile,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let lines: Vec<&[u8]> = source.lines().collect();
    let mut byte_offset = 0usize;
    for (i, line) in lines.iter().enumerate() {
        if skip_last_crlf_line(source, &lines, i) { break; }
        if check_one_crlf(cop, source, line, byte_offset, diagnostics, corrections) { break; }
        byte_offset += line.len() + 1;
    }
}

impl Cop for EndOfLine {
    fn name(&self) -> &'static str { "Layout/EndOfLine" }
    fn supports_autocorrect(&self) -> bool { true }

    fn uses_line_phase(&self) -> bool {
        true
    }

    fn check_lines(
        &self, source: &SourceFile, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "native");
        let want_crlf = style == "crlf" || (style == "native" && cfg!(windows));
        if want_crlf {
            check_crlf(self, source, diagnostics, &mut corrections);
        } else {
            check_lf(self, source, diagnostics, &mut corrections);
        }
    }
}
