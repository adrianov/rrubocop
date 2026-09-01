//! Gemspec/OrderedDependencies — alphabetical gem dependency groups.

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

use super::ordered_dependencies_group::{
    flush_group, line_offsets, try_add_dep, DepEntry,
};

/// ## Corpus investigation (2026-03-03)
///
/// Corpus oracle (run 22651309591) reported FP=0, FN=0. 100% conformance.
pub struct OrderedDependencies;

impl Cop for OrderedDependencies {
    fn name(&self) -> &'static str {
        "Gemspec/OrderedDependencies"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*.gemspec"]
    }

    fn uses_line_phase(&self) -> bool {
        true
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let treat_comments = config.get_bool("TreatCommentsAsGroupSeparators", true);
        let consider_punct = config.get_bool("ConsiderPunctuation", false);
        let bytes = source.as_bytes();
        let offsets = line_offsets(source);
        let mut current_method: Option<String> = None;
        let mut group: Vec<DepEntry> = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            handle_line(
                self,
                source,
                bytes,
                line,
                line_idx,
                offsets[line_idx],
                treat_comments,
                consider_punct,
                &mut current_method,
                &mut group,
                diagnostics,
                &mut corrections,
            );
        }
        flush_group(
            &mut group,
            diagnostics,
            source,
            self,
            &mut corrections,
            bytes,
        );
    }
}

fn handle_line(
    cop: &OrderedDependencies,
    source: &SourceFile,
    bytes: &[u8],
    line: &[u8],
    line_idx: usize,
    range: (usize, usize),
    treat_comments: bool,
    consider_punct: bool,
    current_method: &mut Option<String>,
    group: &mut Vec<DepEntry>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<crate::correction::Correction>>,
) {
    let Ok(line_str) = std::str::from_utf8(line) else {
        reset_group(group, diagnostics, source, cop, corrections, bytes, current_method);
        return;
    };
    let trimmed = line_str.trim();
    if trimmed.is_empty() || (trimmed.starts_with('#') && treat_comments) {
        reset_group(group, diagnostics, source, cop, corrections, bytes, current_method);
        return;
    }
    if trimmed.starts_with('#') {
        return;
    }
    let (line_start, line_end) = range;
    if !try_add_dep(
        line_str,
        line_idx,
        line_start,
        line_end,
        current_method,
        group,
        |g| flush_group(g, diagnostics, source, cop, corrections, bytes),
        consider_punct,
    ) {
        reset_group(group, diagnostics, source, cop, corrections, bytes, current_method);
    }
}

fn reset_group(
    group: &mut Vec<DepEntry>,
    diagnostics: &mut Vec<Diagnostic>,
    source: &SourceFile,
    cop: &OrderedDependencies,
    corrections: &mut Option<&mut Vec<crate::correction::Correction>>,
    bytes: &[u8],
    current_method: &mut Option<String>,
) {
    flush_group(group, diagnostics, source, cop, corrections, bytes);
    *current_method = None;
}
