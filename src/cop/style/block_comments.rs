//! Style/BlockComments — (breadth-first tree-sitter port).

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BlockComments;

impl Cop for BlockComments {
    fn name(&self) -> &'static str {
        "Style/BlockComments"
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        for (idx, line) in source.lines().enumerate() {
            let s = String::from_utf8_lossy(line);
            if s.trim_start().starts_with("=begin") {
                let (line_n, col) =
                    source.offset_to_line_col(source.line_start(idx + 1).unwrap_or(0));
                diagnostics.push(self.diagnostic(
                    source,
                    line_n,
                    col,
                    "Do not use block comments.".to_string(),
                ));
            }
        }
    }
}
