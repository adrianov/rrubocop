//! Layout/InitialIndentation.

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct InitialIndentation;

fn effective_line(i: usize, line: &[u8]) -> (&[u8], usize) {
    if i == 0 && line.starts_with(&[0xEF, 0xBB, 0xBF]) {
        (&line[3..], 3)
    } else {
        (line, 0)
    }
}

fn is_code_line(effective: &[u8]) -> bool {
    match effective.iter().find(|&&b| b != b' ' && b != b'\t' && b != b'\r') {
        None | Some(b'#') => false,
        _ => true,
    }
}

fn leading_ws(effective: &[u8]) -> Option<usize> {
    if effective.first() != Some(&b' ') && effective.first() != Some(&b'\t') {
        return None;
    }
    Some(effective.iter().take_while(|&&b| b == b' ' || b == b'\t').count())
}

fn first_indent_offense(source: &SourceFile) -> Option<(usize, usize)> {
    for (i, line) in source.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let (effective, bom) = effective_line(i, line);
        if !is_code_line(effective) {
            continue;
        }
        let ws_len = leading_ws(effective)?;
        let start = source.line_start(i + 1)?;
        return Some((start + bom, ws_len));
    }
    None
}

impl Cop for InitialIndentation {
    fn name(&self) -> &'static str {
        "Layout/InitialIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn uses_line_phase(&self) -> bool {
        true
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        let Some((off, ws_len)) = first_indent_offense(source) else {
            return;
        };
        report::report_fix(
            self,
            source,
            off,
            "Indentation of first line in file detected.".into(),
            diagnostics,
            &mut corrections,
            off,
            off + ws_len,
            String::new(),
        );
    }
}
