use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::Diagnostic;
use crate::formatter::Formatter;

pub struct TextFormatter;

impl Formatter for TextFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write) {
        let file_count = files.len();
        for d in diagnostics {
            let _ = writeln!(out, "{d}");
        }
        let offense_word = if diagnostics.len() == 1 {
            "offense"
        } else {
            "offenses"
        };
        let file_word = if file_count == 1 { "file" } else { "files" };
        let corrected_count = diagnostics.iter().filter(|d| d.corrected).count();
        if corrected_count > 0 {
            let corrected_word = if corrected_count == 1 {
                "offense"
            } else {
                "offenses"
            };
            let _ = writeln!(
                out,
                "\n{file_count} {file_word} inspected, {} {offense_word} detected, {corrected_count} {corrected_word} corrected",
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
}
