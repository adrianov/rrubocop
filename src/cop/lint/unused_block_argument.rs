use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::model::{self, IntroKind, ScopeKind};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// Lint/UnusedBlockArgument — unused block params.
pub struct UnusedBlockArgument;

impl Cop for UnusedBlockArgument {
    fn name(&self) -> &'static str {
        "Lint/UnusedBlockArgument"
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
            if scope.kind != ScopeKind::Block {
                continue;
            }
            for (name, entry) in &scope.entries {
                if entry.intro_kind != IntroKind::Binding {
                    continue;
                }
                if name.starts_with('_') {
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
                    format!(
                        "Unused block argument - `{name}`. If it's necessary, use `_{name}` as an argument name to indicate that it won't be used."
                    ),
                ));
            }
        }
    }
}
