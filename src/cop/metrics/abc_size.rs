//! Metrics/AbcSize — wraps existing tree-sitter ABC calculator.

use crate::abc;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct AbcSize;

impl Cop for AbcSize {
    fn name(&self) -> &'static str {
        "Metrics/AbcSize"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn needs_file_model(&self) -> bool {
        true
    }

    fn check_file_model(
        &self,
        source: &SourceFile,
        file_model: &crate::model::FileModel<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let max = config.get_f64("Max", 17.0);
        for offense in abc::all_scores(file_model) {
            if offense.score > max {
                let msg = format!(
                    "Assignment Branch Condition size for `{}` is too high. [{} {}/{}]",
                    offense.name,
                    offense.vector,
                    abc::g4(offense.score),
                    abc::g4(max)
                );
                diagnostics.push(self.diagnostic(source, offense.line, offense.column, msg));
            }
        }
    }
}
