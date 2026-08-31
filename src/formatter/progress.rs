use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::{smart_path, Diagnostic, Severity};
use crate::formatter::text::write_summary;
use crate::formatter::Formatter;

/// RuboCop progress formatter: Inspecting header, marks, then clang offenses.
pub struct ProgressFormatter;

impl Formatter for ProgressFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write) {
        write_inspecting(files.len(), out);
        write_marks(diagnostics, files, out);
        write_offense_block(diagnostics, out);
        write_summary(diagnostics, files.len(), out);
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

fn write_marks(diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write) {
    let worst = worst_severity(diagnostics);
    for f in files {
        let mark = worst
            .get(&smart_path(&f.to_string_lossy()))
            .map_or('.', |s| s.letter());
        let _ = write!(out, "{mark}");
    }
    let _ = writeln!(out);
}

fn write_offense_block(diagnostics: &[Diagnostic], out: &mut dyn Write) {
    if diagnostics.is_empty() {
        return;
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Offenses:");
    let _ = writeln!(out);
    for d in diagnostics {
        let _ = writeln!(out, "{d}");
    }
}
