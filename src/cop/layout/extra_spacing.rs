//! Layout/ExtraSpacing.

use tree_sitter::Tree;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct ExtraSpacing;

fn skip_indent(line: &[u8]) -> usize {
    let mut i = 0;
    while i < line.len() && matches!(line[i], b' ' | b'\t') {
        i += 1;
    }
    i
}

fn line_at(source: &SourceFile, line: usize) -> Option<&[u8]> {
    let start = source.line_start(line)?;
    let bytes = source.as_bytes();
    let mut end = start;
    while end < bytes.len() && bytes[end] != b'\n' {
        end += 1;
    }
    Some(&bytes[start..end])
}

/// RuboCop AllowForAlignment: same column has a non-space on a nearby line.
fn aligned_elsewhere(source: &SourceFile, line: usize, col: usize) -> bool {
    for delta in [-5isize, -4, -3, -2, -1, 1, 2, 3, 4, 5] {
        let other = line as isize + delta;
        if other < 1 {
            continue;
        }
        let Some(bytes) = line_at(source, other as usize) else {
            continue;
        };
        if bytes.len() <= col {
            continue;
        }
        if bytes[col] != b' ' && bytes[col] != b'\t' {
            return true;
        }
    }
    false
}

fn should_flag(
    source: &SourceFile,
    allow_aligned: bool,
    line_no: usize,
    line: &[u8],
    _start: usize,
    end: usize,
) -> bool {
    // Never flag spaces just before a trailing comment.
    if line.get(end).copied() == Some(b'#') {
        return false;
    }
    if !allow_aligned {
        return true;
    }
    // Aligned padding: token after spaces shares its column with another line.
    !aligned_elsewhere(source, line_no, end)
}

fn flag_run(
    cop: &dyn Cop,
    source: &SourceFile,
    abs: usize,
    run: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    report::report_fix(
        cop,
        source,
        abs,
        "Unnecessary spacing detected.".into(),
        diagnostics,
        corrections,
        abs,
        abs + run,
        " ".into(),
    );
}

fn scan_line(
    cop: &dyn Cop,
    source: &SourceFile,
    code_map: &CodeMap,
    allow_aligned: bool,
    line_no: usize,
    offset: usize,
    line: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut i = skip_indent(line);
    while i < line.len() {
        if line[i] != b' ' {
            i += 1;
            continue;
        }
        let start = i;
        while i < line.len() && line[i] == b' ' {
            i += 1;
        }
        let run = i - start;
        if run < 2 {
            continue;
        }
        let abs = offset + start;
        if code_map.covers(abs) {
            continue;
        }
        if should_flag(source, allow_aligned, line_no, line, start, i) {
            flag_run(cop, source, abs, run, diagnostics, corrections);
        }
    }
}

impl Cop for ExtraSpacing {
    fn name(&self) -> &'static str {
        "Layout/ExtraSpacing"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn uses_source_phase(&self) -> bool {
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
        let _ = tree;
        let allow_aligned = config.get_bool("AllowForAlignment", true);
        let mut offset = 0usize;
        let mut line_no = 1usize;
        for line in source.lines() {
            scan_line(
                self,
                source,
                code_map,
                allow_aligned,
                line_no,
                offset,
                line,
                diagnostics,
                &mut corrections,
            );
            offset += line.len() + 1;
            line_no += 1;
        }
    }
}
