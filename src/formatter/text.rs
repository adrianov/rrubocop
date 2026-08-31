use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::Diagnostic;
use crate::formatter::color::Color;
use crate::formatter::Formatter;

pub struct TextFormatter {
    pub color: Color,
}

impl Formatter for TextFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write) {
        for d in diagnostics {
            let _ = writeln!(out, "{}", d.render(self.color));
        }
        write_summary(self.color, diagnostics, files.len(), out);
    }
}

fn noun(n: usize, one: &'static str, many: &'static str) -> &'static str {
    if n == 1 { one } else { many }
}

pub(crate) fn write_summary(
    color: Color,
    diagnostics: &[Diagnostic],
    file_count: usize,
    out: &mut dyn Write,
) {
    let n = diagnostics.len();
    let offense_text = format!("{n} {}", noun(n, "offense", "offenses"));
    let offenses = if n == 0 {
        color.green(&offense_text)
    } else {
        color.red(&offense_text)
    };
    let files = format!("{file_count} {}", noun(file_count, "file", "files"));
    let corrected = diagnostics.iter().filter(|d| d.corrected).count();
    if corrected == 0 {
        let _ = writeln!(out, "\n{files} inspected, {offenses} detected");
        return;
    }
    let corr_text = format!("{corrected} {}", noun(corrected, "offense", "offenses"));
    let corr = if corrected == n {
        color.green(&corr_text)
    } else {
        color.cyan(&corr_text)
    };
    let _ = writeln!(
        out,
        "\n{files} inspected, {offenses} detected, {corr} corrected"
    );
}
