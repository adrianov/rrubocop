use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::Diagnostic;
use crate::formatter::Formatter;

pub struct TextFormatter;

impl Formatter for TextFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write) {
        for d in diagnostics {
            let _ = writeln!(out, "{d}");
        }
        write_summary(diagnostics, files.len(), out);
    }
}

pub(crate) fn write_summary(diagnostics: &[Diagnostic], file_count: usize, out: &mut dyn Write) {
    let offense_word = if diagnostics.len() == 1 {
        "offense"
    } else {
        "offenses"
    };
    let file_word = if file_count == 1 { "file" } else { "files" };
    let corrected = diagnostics.iter().filter(|d| d.corrected).count();
    if corrected > 0 {
        let corrected_word = if corrected == 1 {
            "offense"
        } else {
            "offenses"
        };
        let _ = writeln!(
            out,
            "\n{file_count} {file_word} inspected, {} {offense_word} detected, {corrected} {corrected_word} corrected",
            diagnostics.len(),
        );
    } else {
        let _ = writeln!(
            out,
            "\n{file_count} {file_word} inspected, {} {offense_word} detected",
            diagnostics.len(),
        );
    }
}
