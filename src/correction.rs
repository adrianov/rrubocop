//! Source-level autocorrect edits (adapted from nitrocop).

use crate::diagnostic::Diagnostic;

#[derive(Debug, Clone)]
pub struct Correction {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub cop_name: &'static str,
    pub cop_index: usize,
}

pub struct CorrectionSet {
    corrections: Vec<Correction>,
    /// Other cops that emitted the same edit and were dropped as duplicates.
    aliases: Vec<Vec<&'static str>>,
    /// Edits dropped for overlapping an accepted one.
    conflicted: Vec<Correction>,
}

impl CorrectionSet {
    pub fn from_vec(mut raw: Vec<Correction>) -> Self {
        raw.sort_by(|a, b| a.start.cmp(&b.start).then(a.cop_index.cmp(&b.cop_index)));
        let mut set = Self {
            corrections: Vec::with_capacity(raw.len()),
            aliases: Vec::with_capacity(raw.len()),
            conflicted: Vec::new(),
        };
        for c in raw {
            set.add(c);
        }
        set
    }

    /// Accept `c`, merge it into an identical edit, or drop it on overlap.
    fn add(&mut self, c: Correction) {
        if note_same(&self.corrections, &mut self.aliases, &c) {
            return;
        }
        if self
            .corrections
            .last()
            .is_some_and(|last| c.start < last.end)
        {
            self.conflicted.push(c);
            return;
        }
        self.corrections.push(c);
        self.aliases.push(Vec::new());
    }

    pub fn accepted(&self) -> &[Correction] {
        &self.corrections
    }

    pub fn apply(&self, source: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(source.len());
        let mut cursor = 0;
        for c in &self.corrections {
            if c.start > cursor {
                result.extend_from_slice(&source[cursor..c.start]);
            }
            result.extend_from_slice(c.replacement.as_bytes());
            cursor = c.end;
        }
        if cursor < source.len() {
            result.extend_from_slice(&source[cursor..]);
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.corrections.is_empty()
    }

    pub fn len(&self) -> usize {
        self.corrections.len()
    }

    /// Any accepted edit of `cop` when none of its edits conflicted; otherwise one
    /// overlapping `span`, or else the offense owns the nearest `cop` edit not
    /// before it (edits may lie past the highlighted first line of a multi-line
    /// offense), which must be accepted.
    fn credits(&self, cop: &str, span: (usize, usize)) -> bool {
        let (own, dropped) = self.edits_of(cop);
        if dropped.is_empty() || own.iter().any(|e| correction_covers(e, span)) {
            return !own.is_empty();
        }
        nearest_is_accepted(&own, &dropped, span.0)
    }

    /// Accepted edits of `cop` (identical-edit aliases included) and its dropped ones.
    fn edits_of(&self, cop: &str) -> (Vec<&Correction>, Vec<&Correction>) {
        (
            self.corrections
                .iter()
                .zip(&self.aliases)
                .filter(|(edit, names)| edit.cop_name == cop || names.contains(&cop))
                .map(|(edit, _)| edit)
                .collect(),
            self.conflicted
                .iter()
                .filter(|c| c.cop_name == cop)
                .collect(),
        )
    }
}

/// Clear `corrected` on diagnostics whose cop edit was dropped or not applied.
pub fn reconcile_corrected(
    diagnostics: &mut [Diagnostic],
    source: &crate::parse::source::SourceFile,
    set: &CorrectionSet,
) {
    if set.is_empty() {
        for d in diagnostics.iter_mut().filter(|d| d.corrected) {
            d.corrected = false;
        }
        return;
    }
    for d in diagnostics.iter_mut().filter(|d| d.corrected) {
        let Some(off) = source.line_col_to_offset(d.location.line, d.location.column) else {
            d.corrected = false;
            continue;
        };
        // The edit may sit anywhere in the highlight (`&.` inside a whole `if`).
        let last = off + d.highlight_length.saturating_sub(1);
        if !set.credits(&d.cop_name, (off, last)) {
            d.corrected = false;
        }
    }
}

fn note_same(accepted: &[Correction], aliases: &mut [Vec<&'static str>], c: &Correction) -> bool {
    let Some(i) = accepted
        .iter()
        .position(|a| a.start == c.start && a.end == c.end && a.replacement == c.replacement)
    else {
        return false;
    };
    let name = c.cop_name;
    if name != accepted[i].cop_name && !aliases[i].contains(&name) {
        aliases[i].push(name);
    }
    true
}

/// The first edit not ending before `from` is in `accepted`, not `dropped`.
fn nearest_is_accepted(accepted: &[&Correction], dropped: &[&Correction], from: usize) -> bool {
    let first = |edits: &[&Correction]| {
        edits
            .iter()
            .filter(|c| c.end.max(c.start) >= from)
            .map(|c| c.start)
            .min()
    };
    first(accepted).is_some_and(|ok| first(dropped).is_none_or(|bad| ok < bad))
}

fn correction_covers(c: &Correction, (first, last): (usize, usize)) -> bool {
    c.start <= last && first <= c.end.max(c.start)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::diagnostic::{Diagnostic, Location, Severity};
    use crate::parse::source::SourceFile;

    fn edit(start: usize, end: usize, replacement: &str, cop: &'static str) -> Correction {
        Correction {
            start,
            end,
            replacement: replacement.into(),
            cop_name: cop,
            cop_index: 0,
        }
    }

    fn diag(cop: &str, column: usize) -> Diagnostic {
        Diagnostic {
            path: "a.rb".into(),
            cop_name: cop.into(),
            message: "m".into(),
            location: Location { line: 1, column },
            severity: Severity::Convention,
            corrected: true,
            correctable: true,
            source_line: String::new(),
            highlight_length: 1,
        }
    }

    #[test]
    fn identical_edits_keep_both_cops_corrected() {
        let source = SourceFile::from_bytes(Path::new("a.rb"), b"def foo(&block)\nend\n".to_vec());
        let set = CorrectionSet::from_vec(vec![
            edit(8, 14, "&", "Naming/BlockForwarding"),
            edit(8, 14, "&", "Style/ArgumentsForwarding"),
        ]);
        assert_eq!(set.len(), 1);
        assert_eq!(set.apply(source.as_bytes()), b"def foo(&)\nend\n");
        let mut diags = vec![
            diag("Naming/BlockForwarding", 8),
            diag("Style/ArgumentsForwarding", 8),
        ];
        reconcile_corrected(&mut diags, &source, &set);
        assert!(diags.iter().all(|d| d.corrected), "{diags:?}");
    }

    #[test]
    fn conflicting_overlap_does_not_credit_the_dropped_cop() {
        let source = SourceFile::from_bytes(Path::new("a.rb"), b"foo  \n".to_vec());
        let set = CorrectionSet::from_vec(vec![
            edit(3, 5, "", "Layout/Other"),
            edit(3, 5, " ", "Layout/TrailingWhitespace"),
        ]);
        let mut diags = vec![diag("Layout/TrailingWhitespace", 4)];
        reconcile_corrected(&mut diags, &source, &set);
        assert!(!diags[0].corrected);
    }

    #[test]
    fn credits_edits_away_from_the_offense_start() {
        let cop = "Rails/SafeNavigationWithBlank";
        let source = SourceFile::from_bytes(Path::new("a.rb"), b"x if foo&.blank?\n".to_vec());
        let mut diags = vec![diag(cop, 0)];
        reconcile_corrected(
            &mut diags,
            &source,
            &CorrectionSet::from_vec(vec![edit(8, 10, ".", cop)]),
        );
        assert!(diags[0].corrected);
    }

    #[test]
    fn after_a_conflict_the_nearest_edit_decides() {
        let cop = "Rails/SafeNavigationWithBlank";
        let source = SourceFile::from_bytes(Path::new("a.rb"), b"x if foo&.blank?\n".to_vec());
        let mut diags = vec![diag(cop, 0)];
        // The offense owns the nearest edit of its cop: dropped here.
        let conflict = CorrectionSet::from_vec(vec![
            edit(7, 9, "", "Other/Cop"),
            edit(8, 10, ".", cop),
            edit(12, 14, "", cop),
        ]);
        reconcile_corrected(&mut diags, &source, &conflict);
        assert!(!diags[0].corrected);
        // An accepted edit inside the highlight still counts.
        diags[0].corrected = true;
        diags[0].highlight_length = 16;
        reconcile_corrected(&mut diags, &source, &conflict);
        assert!(diags[0].corrected);
        // A later offense of the same cop owns the accepted edit after the dropped one.
        let mut later = vec![diag(cop, 11)];
        reconcile_corrected(&mut later, &source, &conflict);
        assert!(later[0].corrected);
    }
}
