use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// Lint/OrderedMagicComments — encoding before other magic comments.
pub struct OrderedMagicComments;

fn is_encoding(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("encoding:") || lower.contains("coding:") || lower.contains("encoding =")
}

fn is_other_magic(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("frozen_string_literal") || lower.contains("shareable_constant_value")
}

fn first_disorder(source: &SourceFile) -> Option<usize> {
    let mut encoding_seen = false;
    let mut other_line = None;
    for (i, line) in source.lines().enumerate() {
        let s = String::from_utf8_lossy(line);
        let t = s.trim();
        if t.is_empty() {
            continue;
        }
        if !t.starts_with('#') {
            break;
        }
        if is_encoding(t) {
            encoding_seen = true;
            if let Some(line_no) = other_line {
                return Some(line_no);
            }
        } else if is_other_magic(t) && !encoding_seen {
            other_line = Some(i + 1);
        }
    }
    None
}

impl Cop for OrderedMagicComments {
    fn name(&self) -> &'static str {
        "Lint/OrderedMagicComments"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn uses_source_phase(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        _tree: &tree_sitter::Tree,
        _code_map: &CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(line) = first_disorder(source) else {
            return;
        };
        diagnostics.push(self.diagnostic(
            source,
            line,
            0,
            "The encoding magic comment should precede all other magic comments.".to_string(),
        ));
    }
}
