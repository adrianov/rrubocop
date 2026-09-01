mod audit;
mod directives;

use std::collections::HashMap;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

use audit::audit_redundant_disables;
use directives::{cop_names, disable_marker};

/// Lint/RedundantCopDisableDirective — disable comments that suppress no offenses.
pub struct RedundantCopDisableDirective;

fn dup_cop_column(line: &str, cop: &str, occurrence: usize) -> usize {
    let lower = line.to_ascii_lowercase();
    let needle = cop.to_ascii_lowercase();
    let mut found = 0usize;
    let mut start = 0usize;
    while let Some(pos) = lower[start..].find(&needle) {
        let col = start + pos;
        found += 1;
        if found == occurrence {
            return col;
        }
        start = col + needle.len();
    }
    0
}

fn report_dups(
    cop: &RedundantCopDisableDirective,
    source: &SourceFile,
    line_no: usize,
    cops: &[String],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let line = source.line_text(line_no).unwrap_or("");
    let mut nth = HashMap::<&str, usize>::new();
    for name in cops {
        let n = nth.entry(name.as_str()).or_insert(0);
        *n += 1;
        if *n == 1 {
            continue;
        }
        diagnostics.push(cop.diagnostic(
            source,
            line_no,
            dup_cop_column(line, name, *n),
            format!("Unnecessary disabling of `{name}`."),
        ));
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
            let Some((_, rest)) = disable_marker(&s) else {
                continue;
            };
            report_dups(self, source, i + 1, &cop_names(rest), diagnostics);
        }
    }

    fn audit_after_cops(
        &self,
        source: &SourceFile,
        offenses: &[Diagnostic],
        active: &[(&dyn Cop, &CopConfig)],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        audit_redundant_disables(self, source, offenses, active, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Severity};

    crate::cop_fixture_tests!(
        RedundantCopDisableDirective,
        "cops/lint/redundant_cop_disable_directive"
    );

    struct NamedCop(&'static str);

    impl Cop for NamedCop {
        fn name(&self) -> &'static str {
            self.0
        }

        fn uses_source_phase(&self) -> bool {
            true
        }
    }

    fn active_cops(names: &[&'static str]) -> (Vec<NamedCop>, Vec<CopConfig>) {
        let cops: Vec<NamedCop> = names.iter().map(|n| NamedCop(n)).collect();
        let cfgs = vec![CopConfig::default(); names.len()];
        (cops, cfgs)
    }

    fn active_refs<'a>(
        cops: &'a [NamedCop],
        cfgs: &'a [CopConfig],
    ) -> Vec<(&'a dyn Cop, &'a CopConfig)> {
        cops.iter()
            .zip(cfgs.iter())
            .map(|(c, cfg)| (c as &dyn Cop, cfg))
            .collect()
    }

    fn offense(cop: &str, line: usize) -> Diagnostic {
        Diagnostic {
            path: "test.rb".into(),
            location: Location { line, column: 0 },
            severity: Severity::Convention,
            cop_name: cop.into(),
            message: String::new(),
            corrected: false,
            correctable: false,
            source_line: String::new(),
            highlight_length: 1,
        }
    }

    #[test]
    fn duplicate_cop_in_disable() {
        let source = SourceFile::from_bytes(
            "test.rb",
            b"x = 1 # rubocop:disable Style/StringLiterals, Style/StringLiterals\n".to_vec(),
        );
        let tree = crate::parse::parse_ruby(&source).unwrap();
        let code_map =
            crate::parse::codemap::CodeMap::from_tree(tree.root_node(), source.as_bytes());
        let cop = RedundantCopDisableDirective;
        let mut diags = Vec::new();
        cop.check_source(
            &source,
            &tree,
            &code_map,
            &CopConfig::default(),
            &mut diags,
            None,
        );
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("Style/StringLiterals"));
    }

    #[test]
    fn needed_disable_not_reported() {
        let source =
            SourceFile::from_bytes("test.rb", b"puts 'x' # rubocop:disable Rails/Output\n".to_vec());
        let offenses = vec![offense("Rails/Output", 1)];
        let cop = RedundantCopDisableDirective;
        let mut diags = Vec::new();
        let (cops, cfgs) = active_cops(&["Rails/Output"]);
        let active = active_refs(&cops, &cfgs);
        audit_redundant_disables(&cop, &source, &offenses, &active, &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn redundant_block_disable() {
        let source = SourceFile::from_bytes(
            "test.rb",
            b"# rubocop:disable Rails/Output\nRails.logger.debug 'x'\n# rubocop:enable Rails/Output\n"
                .to_vec(),
        );
        let cop = RedundantCopDisableDirective;
        let mut diags = Vec::new();
        let (cops, cfgs) = active_cops(&["Rails/Output"]);
        let active = active_refs(&cops, &cfgs);
        audit_redundant_disables(&cop, &source, &[], &active, &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn redundant_multi_cop_block_disable() {
        let source = SourceFile::from_bytes(
            "test.rb",
            b"# rubocop:disable Rails/Output, Layout/LineLength\nx = 1\n# rubocop:enable Rails/Output, Layout/LineLength\n"
                .to_vec(),
        );
        let cop = RedundantCopDisableDirective;
        let mut diags = Vec::new();
        let (cops, cfgs) = active_cops(&["Rails/Output", "Layout/LineLength"]);
        let active = active_refs(&cops, &cfgs);
        audit_redundant_disables(&cop, &source, &[], &active, &mut diags);
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn partial_block_enable_audits_each_cop_once() {
        let source = SourceFile::from_bytes(
            "test.rb",
            b"# rubocop:disable Rails/Output, Layout/LineLength\nx = 1\n# rubocop:enable Rails/Output\n# rubocop:enable Layout/LineLength\n"
                .to_vec(),
        );
        let cop = RedundantCopDisableDirective;
        let mut diags = Vec::new();
        let (cops, cfgs) = active_cops(&["Rails/Output", "Layout/LineLength"]);
        let active = active_refs(&cops, &cfgs);
        audit_redundant_disables(&cop, &source, &[], &active, &mut diags);
        assert_eq!(diags.len(), 2);
        assert!(diags.iter().any(|d| d.message.contains("Rails/Output")));
        assert!(diags.iter().any(|d| d.message.contains("Layout/LineLength")));
    }

    #[test]
    fn enable_all_closes_every_open_block() {
        let source = SourceFile::from_bytes(
            "test.rb",
            b"# rubocop:disable Rails/Output, Layout/LineLength\nx = 1\n# rubocop:enable all\n"
                .to_vec(),
        );
        let cop = RedundantCopDisableDirective;
        let mut diags = Vec::new();
        let (cops, cfgs) = active_cops(&["Rails/Output", "Layout/LineLength"]);
        let active = active_refs(&cops, &cfgs);
        audit_redundant_disables(&cop, &source, &[], &active, &mut diags);
        assert_eq!(diags.len(), 2);
        assert!(diags.iter().all(|d| d.location.line == 1));
    }

    #[test]
    fn skip_disable_for_unrun_cop() {
        let source = SourceFile::from_bytes(
            "test.rb",
            b"User.update_all(active: false) # rubocop:disable Rails/SkipsModelValidations\n"
                .to_vec(),
        );
        let cop = RedundantCopDisableDirective;
        let mut diags = Vec::new();
        audit_redundant_disables(&cop, &source, &[], &[], &mut diags);
        assert!(diags.is_empty());
    }
}
