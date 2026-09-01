//! Lint/UselessAssignment — write never read (uses model).

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::model::{self, IntroKind};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct UselessAssignment;

impl Cop for UselessAssignment {
    fn name(&self) -> &'static str {
        "Lint/UselessAssignment"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &tree_sitter::Tree,
        _code_map: &CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let fm = model::build(source.as_bytes(), tree.clone());
        for scope in &fm.scopes {
            for (name, entry) in &scope.entries {
                if name.starts_with('_') {
                    continue;
                }
                if entry.intro_kind != IntroKind::Assign {
                    continue;
                }
                if !entry.reads.is_empty() {
                    continue;
                }
                let (line, col) = fm.line_col(entry.intro_byte);
                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    col,
                    format!("Useless assignment to variable - `{name}`."),
                ));
            }
        }
    }
}
