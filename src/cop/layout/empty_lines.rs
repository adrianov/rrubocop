//! Layout/EmptyLines — empty lines around code (simplified: consecutive blanks > 1).

use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLines;

fn is_blank(line: &[u8]) -> bool {
    line.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r')
}

fn report_extra(
    cop: &dyn Cop, source: &SourceFile, lines: &[&[u8]], start_line: usize, start_byte: usize,
    remove_end: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut diag = cop.diagnostic(source, start_line + 2, 0, "Extra blank line detected.".into());
    if let Some(corr) = corrections {
        let keep_end = start_byte + lines[start_line].len() + 1;
        if remove_end > keep_end {
            corr.push(Correction {
                start: keep_end, end: remove_end, replacement: String::new(),
                cop_name: cop.name(), cop_index: 0,
            });
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

fn on_nonblank(
    cop: &dyn Cop, source: &SourceFile, lines: &[&[u8]],
    blank_count: usize, run: Option<(usize, usize)>, byte_offset: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if blank_count <= 1 { return; }
    let Some((start_line, start_byte)) = run else { return; };
    report_extra(cop, source, lines, start_line, start_byte, byte_offset, diagnostics, corrections);
}

impl Cop for EmptyLines {
    fn name(&self) -> &'static str { "Layout/EmptyLines" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_lines(
        &self, source: &SourceFile, _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let lines: Vec<&[u8]> = source.lines().collect();
        let mut byte_offset = 0usize;
        let mut blank_run_start: Option<(usize, usize)> = None;
        let mut blank_count = 0usize;
        for (i, line) in lines.iter().enumerate() {
            if is_blank(line) {
                if blank_count == 0 { blank_run_start = Some((i, byte_offset)); }
                blank_count += 1;
            } else {
                on_nonblank(
                    self, source, &lines, blank_count, blank_run_start, byte_offset,
                    diagnostics, &mut corrections,
                );
                blank_count = 0;
                blank_run_start = None;
            }
            byte_offset += line.len() + 1;
        }
    }
}
