use std::io::Write;

use crate::diagnostic::Diagnostic;
use crate::formatter::color::Color;
use crate::formatter::Formatter;

pub struct TextFormatter {
    pub color: Color,
}

impl Formatter for TextFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[std::path::PathBuf], out: &mut dyn Write) {
        for d in diagnostics {
            let _ = writeln!(out, "{}", d.render(self.color));
        }
        write_summary(self.color, diagnostics, files.len(), out);
    }
}

fn noun(n: usize, one: &'static str, many: &'static str) -> &'static str {
    if n == 1 { one } else { many }
}

fn count_fix_states(diagnostics: &[Diagnostic]) -> (usize, usize) {
    let corrected = diagnostics.iter().filter(|d| d.corrected).count();
    let correctable = diagnostics
        .iter()
        .filter(|d| d.correctable && !d.corrected)
        .count();
    (corrected, correctable)
}

fn colored_offense_count(color: Color, n: usize) -> String {
    let text = format!("{n} {}", noun(n, "offense", "offenses"));
    if n == 0 {
        color.green(&text)
    } else {
        color.red(&text)
    }
}

fn write_with_corrected(
    color: Color,
    files: &str,
    offenses: &str,
    n: usize,
    corrected: usize,
    correctable: usize,
    out: &mut dyn Write,
) {
    let corr_text = format!("{corrected} {}", noun(corrected, "offense", "offenses"));
    let corr = if corrected == n {
        color.green(&corr_text)
    } else {
        color.cyan(&corr_text)
    };
    if correctable == 0 {
        let _ = writeln!(out, "\n{files} inspected, {offenses} detected, {corr} corrected");
        return;
    }
    let more = format!("{correctable} {}", noun(correctable, "offense", "offenses"));
    let _ = writeln!(
        out,
        "\n{files} inspected, {offenses} detected, {corr} corrected, {} autocorrectable",
        color.yellow(&more)
    );
}

pub(crate) fn write_summary(
    color: Color,
    diagnostics: &[Diagnostic],
    file_count: usize,
    out: &mut dyn Write,
) {
    let n = diagnostics.len();
    let offenses = colored_offense_count(color, n);
    let files = format!("{file_count} {}", noun(file_count, "file", "files"));
    let (corrected, correctable) = count_fix_states(diagnostics);
    if corrected == 0 && correctable == 0 {
        let _ = writeln!(out, "\n{files} inspected, {offenses} detected");
    } else if corrected > 0 {
        write_with_corrected(color, &files, &offenses, n, corrected, correctable, out);
    } else {
        let more = format!("{correctable} {}", noun(correctable, "offense", "offenses"));
        let _ = writeln!(
            out,
            "\n{files} inspected, {offenses} detected, {} autocorrectable",
            color.yellow(&more)
        );
    }
}
