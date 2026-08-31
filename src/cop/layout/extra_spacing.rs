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
    while i < line.len() && matches!(line[i], b' ' | b'\t') { i += 1; }
    i
}

fn should_flag(allow_aligned: bool, line: &[u8], start: usize, i: usize) -> bool {
    if !allow_aligned { return true; }
    let before = if start > 0 { line[start - 1] } else { b'\n' };
    let after = line.get(i).copied().unwrap_or(b'\n');
    before != b'\n' && after != b'\n' && before != b'#'
}

fn flag_run(
    cop: &dyn Cop, source: &SourceFile, abs: usize, run: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    report::report_fix(
        cop, source, abs, "Unnecessary spacing detected.".into(),
        diagnostics, corrections, abs, abs + run, " ".into(),
    );
}

fn scan_line(
    cop: &dyn Cop, source: &SourceFile, code_map: &CodeMap, allow_aligned: bool,
    offset: usize, line: &[u8],
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut i = skip_indent(line);
    while i < line.len() {
        if line[i] != b' ' { i += 1; continue; }
        let start = i;
        while i < line.len() && line[i] == b' ' { i += 1; }
        let run = i - start;
        if run < 2 { continue; }
        let abs = offset + start;
        if code_map.covers(abs) { continue; }
        if should_flag(allow_aligned, line, start, i) {
            flag_run(cop, source, abs, run, diagnostics, corrections);
        }
    }
}

impl Cop for ExtraSpacing {
    fn name(&self) -> &'static str { "Layout/ExtraSpacing" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, tree: &Tree, code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = tree;
        let allow_aligned = config.get_bool("AllowForAlignment", true);
        let mut offset = 0usize;
        for line in source.lines() {
            scan_line(
                self, source, code_map, allow_aligned, offset, line,
                diagnostics, &mut corrections,
            );
            offset += line.len() + 1;
        }
    }
}
