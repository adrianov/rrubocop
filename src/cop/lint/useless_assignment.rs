//! Lint/UselessAssignment — write never read (uses model).

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::model::IntroKind;
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

    fn needs_file_model(&self) -> bool {
        true
    }

    fn check_file_model(
        &self,
        source: &SourceFile,
        file_model: &crate::model::FileModel<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        for scope in &file_model.scopes {
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
                let (line, col) = file_model.line_col(entry.intro_byte);
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
