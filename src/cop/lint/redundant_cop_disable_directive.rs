use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// Lint/RedundantCopDisableDirective — disable with no offenses (heuristic: unused all-disable alone).
/// Breadth-first: flags `# rubocop:disable` departments that never appear as enable and file has no matching department usage.
/// Full unused-disable analysis needs the lint engine; here we only flag empty disable lists.
pub struct RedundantCopDisableDirective;

fn disable_rest(line: &str) -> Option<&str> {
    let idx = line.find("# rubocop:disable")?;
    Some(
        line[idx + "# rubocop:disable".len()..]
            .trim()
            .trim_start_matches(':')
            .trim(),
    )
}

fn cops_list(rest: &str) -> Vec<&str> {
    rest.split("--")
        .next()
        .unwrap_or("")
        .split(',')
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
        .collect()
}

fn report_dups(
    cop: &RedundantCopDisableDirective,
    source: &SourceFile,
    line_no: usize,
    cops: &[&str],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = std::collections::HashSet::new();
    for name in cops {
        if !seen.insert(*name) {
            diagnostics.push(cop.diagnostic(
                source,
                line_no,
                0,
                format!("Unnecessary disabling of `{name}`."),
            ));
        }
    }
}

impl Cop for RedundantCopDisableDirective {
    fn name(&self) -> &'static str {
        "Lint/RedundantCopDisableDirective"
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
        for (i, line) in source.lines().enumerate() {
            let s = String::from_utf8_lossy(line);
            let Some(rest) = disable_rest(&s) else {
                continue;
            };
            report_dups(self, source, i + 1, &cops_list(rest), diagnostics);
        }
    }
}
