use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::{smart_path, Diagnostic, Severity};
use crate::formatter::color::Color;
use crate::formatter::text::write_summary;
use crate::formatter::Formatter;

/// RuboCop progress formatter: Inspecting header, marks, then clang offenses.
pub struct ProgressFormatter {
    pub color: Color,
}

impl Formatter for ProgressFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write) {
        write_inspecting(files.len(), out);
        write_marks(self.color, diagnostics, files, out);
        write_offense_block(self.color, diagnostics, out);
        write_summary(self.color, diagnostics, files.len(), out);
    }
}

fn write_inspecting(file_count: usize, out: &mut dyn Write) {
    let word = if file_count == 1 { "file" } else { "files" };
    let _ = writeln!(out, "Inspecting {file_count} {word}");
}

fn worst_severity(diagnostics: &[Diagnostic]) -> HashMap<String, Severity> {
    let mut map = HashMap::new();
    for d in diagnostics {
        let key = smart_path(&d.path);
        map.entry(key)
            .and_modify(|s| {
                if d.severity > *s {
                    *s = d.severity;
                }
            })
            .or_insert(d.severity);
    }
    map
}

fn write_marks(
    color: Color,
    diagnostics: &[Diagnostic],
    files: &[PathBuf],
    out: &mut dyn Write,
) {
    let worst = worst_severity(diagnostics);
    for f in files {
        let mark = match worst.get(&smart_path(&f.to_string_lossy())) {
            Some(s) => color.severity_letter(*s),
            None => color.green("."),
        };
        let _ = write!(out, "{mark}");
    }
    let _ = writeln!(out);
}

fn write_offense_block(color: Color, diagnostics: &[Diagnostic], out: &mut dyn Write) {
    if diagnostics.is_empty() {
        return;
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Offenses:");
    let _ = writeln!(out);
    for d in diagnostics {
        let _ = writeln!(out, "{}", d.render(color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Severity};

    fn sample() -> Diagnostic {
        Diagnostic {
            path: "a.rb".into(),
            location: Location { line: 1, column: 0 },
            severity: Severity::Convention,
            cop_name: "Metrics/AbcSize".into(),
            message: "too high for `m`".into(),
            corrected: false,
        }
    }

    #[test]
    fn progress_colors_mark_and_summary() {
        let fmt = ProgressFormatter {
            color: Color::resolve(Some(true)),
        };
        let mut buf = Vec::new();
        fmt.format_to(&[sample()], &[PathBuf::from("a.rb")], &mut buf);
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\x1b[33mC\x1b[0m"), "{s}");
        assert!(s.contains("\x1b[36ma.rb\x1b[0m"), "{s}");
        assert!(s.contains("\x1b[31m1 offense\x1b[0m"), "{s}");
        assert!(s.contains("\x1b[33mm\x1b[0m"), "{s}");
    }

    #[test]
    fn progress_green_dot_when_clean() {
        let fmt = ProgressFormatter {
            color: Color::resolve(Some(true)),
        };
        let mut buf = Vec::new();
        fmt.format_to(&[], &[PathBuf::from("a.rb")], &mut buf);
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\x1b[32m.\x1b[0m"), "{s}");
        assert!(s.contains("\x1b[32m0 offenses\x1b[0m"), "{s}");
    }
}
