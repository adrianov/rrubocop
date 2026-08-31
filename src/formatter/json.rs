use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::formatter::Formatter;

pub struct JsonFormatter;

#[derive(Serialize)]
struct JsonOutput {
    metadata: Metadata,
    offenses: Vec<Offense>,
}

#[derive(Serialize)]
struct Metadata {
    files_inspected: usize,
    offense_count: usize,
    corrected_count: usize,
}

#[derive(Serialize)]
struct Offense {
    path: String,
    line: usize,
    column: usize,
    severity: String,
    cop_name: String,
    message: String,
    corrected: bool,
}

impl Formatter for JsonFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write) {
        let corrected_count = diagnostics.iter().filter(|d| d.corrected).count();
        let output = JsonOutput {
            metadata: Metadata {
                files_inspected: files.len(),
                offense_count: diagnostics.len(),
                corrected_count,
            },
            offenses: diagnostics
                .iter()
                .map(|d| Offense {
                    path: d.path.clone(),
                    line: d.location.line,
                    column: d.location.column,
                    severity: format!("{}", d.severity),
                    cop_name: d.cop_name.clone(),
                    message: d.message.clone(),
                    corrected: d.corrected,
                })
                .collect(),
        };
        let _ = serde_json::to_writer(&mut *out, &output);
        let _ = writeln!(out);
    }
}
