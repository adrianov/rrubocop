use std::io::Write;
use std::path::{Path, PathBuf};

use crate::diagnostic::Diagnostic;
use crate::formatter::Formatter;

/// RuboCop EmacsStyleFormatter: one machine-parsable line per offense with the
/// absolute path and the raw message; no source line, color, or summary.
pub struct EmacsFormatter;

impl Formatter for EmacsFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], _files: &[PathBuf], out: &mut dyn Write) {
        for d in diagnostics {
            let _ = writeln!(out, "{}", line(d));
        }
    }
}

fn line(d: &Diagnostic) -> String {
    let path = std::path::absolute(Path::new(&d.path)).unwrap_or_else(|_| PathBuf::from(&d.path));
    let status = if d.corrected {
        "[Corrected] "
    } else if d.correctable {
        "[Correctable] "
    } else {
        ""
    };
    format!(
        "{}:{}:{}: {}: {status}{}",
        path.display(),
        d.location.line,
        d.location.column + 1,
        d.severity.letter(),
        d.message.replace('\n', " "),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Severity};

    #[test]
    fn keeps_backticks_and_drops_source_line() {
        let d = Diagnostic {
            path: "/app/a.rb".into(),
            location: Location { line: 3, column: 4 },
            severity: Severity::Convention,
            cop_name: "Style/FrozenStringLiteralComment".into(),
            message: "Style/FrozenStringLiteralComment: Missing `# frozen`.\nmore".into(),
            corrected: false,
            correctable: true,
            source_line: "puts 1".into(),
            highlight_length: 1,
        };
        let mut buf = Vec::new();
        EmacsFormatter.format_to(&[d], &[], &mut buf);
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            "/app/a.rb:3:5: C: [Correctable] Style/FrozenStringLiteralComment: Missing `# frozen`. more\n"
        );
    }
}
