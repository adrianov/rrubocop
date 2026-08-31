use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::model;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// Lint/UnderscorePrefixedVariableName — `_x` that is used.
pub struct UnderscorePrefixedVariableName;

impl Cop for UnderscorePrefixedVariableName {
    fn name(&self) -> &'static str {
        "Lint/UnderscorePrefixedVariableName"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
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
                if !name.starts_with('_') || name.as_ref() == "_" {
                    continue;
                }
                if entry.reads.is_empty() {
                    continue;
                }
                let (line, col) = fm.line_col(entry.intro_byte);
                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    col,
                    "Do not use prefix `_` for a variable that is used.".to_string(),
                ));
            }
        }
    }
}
