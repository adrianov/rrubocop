//! Lint/EmptyFile — empty (0-byte) source files.

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/EmptyFile — flag truly empty files (and comment-only when configured).
pub struct EmptyFile;

fn has_code_line(source: &SourceFile) -> bool {
    source.lines().any(|line| {
        let trimmed = line
            .iter()
            .position(|&b| b != b' ' && b != b'\t' && b != b'\r')
            .map(|start| &line[start..])
            .unwrap_or(&[]);
        !trimmed.is_empty() && !trimmed.starts_with(b"#")
    })
}

fn push_empty(cop: &EmptyFile, source: &SourceFile, diagnostics: &mut Vec<Diagnostic>) {
    diagnostics.push(cop.diagnostic(source, 1, 0, "Empty file detected.".to_string()));
}

impl Cop for EmptyFile {
    fn name(&self) -> &'static str {
        "Lint/EmptyFile"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn uses_line_phase(&self) -> bool {
        true
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if source.as_bytes().is_empty() {
            push_empty(self, source, diagnostics);
            return;
        }
        if !config.get_bool("AllowComments", true) && !has_code_line(source) {
            push_empty(self, source, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_offense() {
        let diags = crate::testutil::run_cop_full(&EmptyFile, b"");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message, "Empty file detected.");
        assert_eq!(diags[0].location.line, 1);
    }

    #[test]
    fn whitespace_only_no_offense() {
        crate::testutil::assert_cop_no_offenses_full(&EmptyFile, b"\n\n");
    }

    #[test]
    fn code_no_offense() {
        crate::testutil::assert_cop_no_offenses_full(&EmptyFile, b"x = 1\n");
    }
}
