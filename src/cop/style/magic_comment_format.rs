//! Style/MagicCommentFormat — (breadth-first tree-sitter port).

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MagicCommentFormat;

impl Cop for MagicCommentFormat {
    fn name(&self) -> &'static str {
        "Style/MagicCommentFormat"
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        for (idx, line) in source.lines().enumerate().take(5) {
            if bad_magic_case(line) {
                let (line_n, col) =
                    source.offset_to_line_col(source.line_start(idx + 1).unwrap_or(0));
                diagnostics.push(self.diagnostic(
                    source,
                    line_n,
                    col,
                    "Use snake_case for magic comment keys.".to_string(),
                ));
            }
        }
    }
}

fn bad_magic_case(line: &[u8]) -> bool {
    let t = String::from_utf8_lossy(line);
    let lower = t.to_ascii_lowercase();
    let is_magic = lower.contains("frozen_string_literal")
        || lower.contains("encoding")
        || lower.contains("coding:");
    is_magic && (t.contains("frozenStringLiteral") || t.contains("FrozenStringLiteral"))
}
