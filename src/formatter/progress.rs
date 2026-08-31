use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::Diagnostic;
use crate::formatter::Formatter;
use crate::formatter::text::TextFormatter;

/// Progress formatter: same summary as text for now (dots can come later).
pub struct ProgressFormatter;

impl Formatter for ProgressFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write) {
        TextFormatter.format_to(diagnostics, files, out);
    }
}
