//! Layout/LineContinuationLeadingSpace.

use tree_sitter::Tree;

use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct LineContinuationLeadingSpace;

fn ends_with_cont(line: &[u8]) -> bool {
    line.ends_with(b"\\")
        || (line.len() >= 2 && line[line.len() - 1] == b'\r' && line[line.len() - 2] == b'\\')
}

fn report_cont(
    cop: &dyn Cop, source: &SourceFile, cont_off: usize, spaces: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (l, c) = source.offset_to_line_col(cont_off);
    let mut diag = cop.diagnostic(
        source, l, c,
        "Do not use more than one space before a line-continued string.".into(),
    );
    if spaces > 1 {
        if let Some(corr) = corrections {
            corr.push(Correction {
                start: cont_off, end: cont_off + spaces, replacement: " ".into(),
                cop_name: cop.name(), cop_index: 0,
            });
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

fn maybe_cont(
    cop: &dyn Cop, source: &SourceFile, code_map: &CodeMap, next: &[u8], cont_off: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if next.first() != Some(&b' ') || !code_map.covers(cont_off) { return; }
    let spaces = next.iter().take_while(|&&b| b == b' ').count();
    report_cont(cop, source, cont_off, spaces, diagnostics, corrections);
}

fn scan_lines(
    cop: &dyn Cop, source: &SourceFile, code_map: &CodeMap,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let lines: Vec<&[u8]> = source.lines().collect();
    let mut offset = 0usize;
    for (idx, line) in lines.iter().enumerate() {
        if ends_with_cont(line) && idx + 1 < lines.len() {
            maybe_cont(cop, source, code_map, lines[idx + 1], offset + line.len() + 1, diagnostics, corrections);
        }
        offset += line.len() + 1;
    }
}

impl Cop for LineContinuationLeadingSpace {
    fn name(&self) -> &'static str { "Layout/LineContinuationLeadingSpace" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, tree: &Tree, code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = (tree, config);
        scan_lines(self, source, code_map, diagnostics, &mut corrections);
    }
}
