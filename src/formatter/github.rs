use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::Diagnostic;
use crate::formatter::Formatter;

pub struct GithubFormatter;

impl Formatter for GithubFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], _files: &[PathBuf], out: &mut dyn Write) {
        for d in diagnostics {
            let _ = writeln!(
                out,
                "::error file={},line={},col={}::[{}] {}",
                d.path,
                d.location.line,
                d.location.column + 1,
                d.cop_name,
                d.message
            );
        }
    }
}
