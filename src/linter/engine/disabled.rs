//! `# rubocop:disable` filtering: drops disabled offenses and the edits they own.

use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::directives;
use crate::parse::source::SourceFile;

/// Drops disabled offenses and their edits.
pub(super) fn filter_directives(
    source: &SourceFile,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<Vec<Correction>>,
) {
    let dirs = directives::parse(&String::from_utf8_lossy(source.as_bytes()));
    if let Some(corrections) = corrections {
        corrections.retain(|c| !dirs.suppresses(c.cop_name, owner_line(source, diagnostics, c)));
    }
    diagnostics.retain(|d| !dirs.suppresses(&d.cop_name, d.location.line));
}

/// Line of the offense owning edit `c`: the nearest same-cop offense before it
/// when that offense spans lines (a modifier `if` whose `&.` sits on its last
/// line), else the edit's own line.
fn owner_line(source: &SourceFile, diagnostics: &[Diagnostic], c: &Correction) -> usize {
    diagnostics
        .iter()
        .filter(|d| d.cop_name == c.cop_name)
        .filter_map(|d| {
            Some((
                source.line_col_to_offset(d.location.line, d.location.column)?,
                d,
            ))
        })
        .filter(|(off, _)| *off <= c.start)
        .max_by_key(|(off, _)| *off)
        .filter(|(_, d)| spans_lines(source, d))
        .map_or_else(
            || source.offset_to_line_col(c.start).0,
            |(_, d)| d.location.line,
        )
}

/// The offense's source line is its real first line plus RuboCop's ` ...` marker.
fn spans_lines(source: &SourceFile, d: &Diagnostic) -> bool {
    d.source_line
        .strip_suffix(" ...")
        .is_some_and(|line| Some(line) == source.line_text(d.location.line))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Severity};

    #[test]
    fn owner_line_needs_the_multiline_marker() {
        let source = SourceFile::from_bytes("a.rb", b"x = 1 ...\ny = 2\n".to_vec());
        let edit = Correction {
            start: 10,
            end: 11,
            replacement: String::new(),
            cop_name: "C/Op",
            cop_index: 0,
        };
        let mut offense = Diagnostic {
            path: "a.rb".into(),
            location: Location { line: 1, column: 0 },
            severity: Severity::Convention,
            cop_name: "C/Op".into(),
            message: String::new(),
            corrected: false,
            correctable: true,
            source_line: "x = 1 ...".into(),
            highlight_length: 1,
        };
        assert_eq!(
            owner_line(&source, std::slice::from_ref(&offense), &edit),
            2
        );
        offense.source_line.push_str(" ...");
        assert_eq!(owner_line(&source, &[offense], &edit), 1);
    }
}
