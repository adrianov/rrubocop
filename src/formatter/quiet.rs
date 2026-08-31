use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::Diagnostic;
use crate::formatter::color::Color;
use crate::formatter::Formatter;

pub struct QuietFormatter {
    pub color: Color,
}

impl Formatter for QuietFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], _files: &[PathBuf], out: &mut dyn Write) {
        for d in diagnostics {
            let _ = writeln!(out, "{}", d.render(self.color));
        }
    }
}
