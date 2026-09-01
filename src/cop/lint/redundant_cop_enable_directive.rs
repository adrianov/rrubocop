use std::collections::HashSet;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// Lint/RedundantCopEnableDirective — enable without matching disable.
pub struct RedundantCopEnableDirective;

fn directive_names(rest: &str) -> Vec<String> {
    let cops = rest
        .trim()
        .trim_start_matches(':')
        .trim()
        .split("--")
        .next()
        .unwrap_or("");
    let names: Vec<String> = cops
        .split(',')
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect();
    if names.is_empty() {
        vec!["all".into()]
    } else {
        names
    }
}

fn config_disabled(config: &CopConfig) -> HashSet<String> {
    match config.options.get("ConfigDisabledCops") {
        Some(serde_yml::Value::Sequence(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => HashSet::new(),
    }
}

fn check_enable(
    cop: &RedundantCopEnableDirective,
    source: &SourceFile,
    disabled: &mut HashSet<String>,
    line: &str,
    line_no: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(rest) = line.split("# rubocop:enable").nth(1) else {
        return;
    };
    if !line.trim_start().starts_with('#') {
        return;
    }
    for name in directive_names(rest) {
        if disabled.remove(&name) || disabled.contains("all") {
            continue;
        }
        diagnostics.push(cop.diagnostic(
            source,
            line_no,
            0,
            format!("Unnecessary enabling of {name}."),
        ));
    }
}

impl Cop for RedundantCopEnableDirective {
    fn name(&self) -> &'static str {
        "Lint/RedundantCopEnableDirective"
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
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // RuboCop seeds disable counts with cops disabled in config.
        let mut disabled = config_disabled(config);
        for (i, line) in source.lines().enumerate() {
            let s = String::from_utf8_lossy(line);
            if let Some(rest) = s.split("# rubocop:disable").nth(1) {
                for name in directive_names(rest) {
                    disabled.insert(name);
                }
            }
            check_enable(self, source, &mut disabled, &s, i + 1, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        RedundantCopEnableDirective,
        "cops/lint/redundant_cop_enable_directive"
    );

    #[test]
    fn config_disabled_enable_ok() {
        let mut config = CopConfig::default();
        config.options.insert(
            "ConfigDisabledCops".into(),
            serde_yml::Value::Sequence(vec![serde_yml::Value::String(
                "Layout/LineLength".into(),
            )]),
        );
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &RedundantCopEnableDirective,
            b"x = 1\n# rubocop:enable Layout/LineLength\n",
            config,
        );
    }
}
