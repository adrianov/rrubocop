use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::Diagnostic;
use crate::formatter::Formatter;

pub struct FilesFormatter;

impl Formatter for FilesFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], _files: &[PathBuf], out: &mut dyn Write) {
        let mut seen = std::collections::BTreeSet::new();
        for d in diagnostics {
            if seen.insert(d.path.clone()) {
                let _ = writeln!(out, "{}", d.path);
            }
        }
    }
}
