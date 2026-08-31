//! Assert fixture expected offenses against a cop run.

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;

use super::parse::{parse_fixture, ExpectedOffense};
use super::run::run_cop_full_internal;

fn format_diagnostics(diags: &[Diagnostic]) -> String {
    diags
        .iter()
        .map(|d| {
            format!(
                "  {}:{}:{} {}: {}",
                d.cop_name, d.location.line, d.location.column, d.severity, d.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_expected(expected: &[ExpectedOffense]) -> String {
    expected
        .iter()
        .map(|e| format!("  {}:{}:{} ?: {}", e.cop_name, e.line, e.column, e.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_one_offense(i: usize, diag: &Diagnostic, exp: &ExpectedOffense) {
    assert_eq!(
        diag.location.line, exp.line,
        "Offense #{} line: expected {} got {}\n  {diag}",
        i + 1,
        exp.line,
        diag.location.line
    );
    assert_eq!(
        diag.location.column, exp.column,
        "Offense #{} column: expected {} got {}\n  {diag}",
        i + 1,
        exp.column,
        diag.location.column
    );
    assert_eq!(diag.cop_name, exp.cop_name, "Offense #{} cop_name", i + 1);
    assert_eq!(
        diag.message, exp.message,
        "Offense #{} message\n  expected: {:?}\n  actual:   {:?}",
        i + 1,
        exp.message,
        diag.message
    );
}

pub fn assert_cop_offenses_full(cop: &dyn Cop, fixture_bytes: &[u8]) {
    assert_cop_offenses_full_with_config(cop, fixture_bytes, CopConfig::default());
}

pub fn assert_cop_offenses_full_with_config(
    cop: &dyn Cop,
    fixture_bytes: &[u8],
    config: CopConfig,
) {
    let parsed = parse_fixture(fixture_bytes);
    let filename = parsed.filename.as_deref().unwrap_or("test.rb");
    let diagnostics = run_cop_full_internal(cop, &parsed.source, config, filename);
    assert_offense_lists(diagnostics, parsed.expected);
}

fn assert_offense_lists(mut diagnostics: Vec<Diagnostic>, mut expected: Vec<ExpectedOffense>) {
    expected.sort_by_key(|e| (e.line, e.column));
    diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    assert_eq!(
        diagnostics.len(),
        expected.len(),
        "Expected {} offense(s) but got {}.\nExpected:\n{}\nActual:\n{}",
        expected.len(),
        diagnostics.len(),
        format_expected(&expected),
        format_diagnostics(&diagnostics),
    );
    for (i, (diag, exp)) in diagnostics.iter().zip(expected.iter()).enumerate() {
        assert_one_offense(i, diag, exp);
    }
}

pub fn assert_cop_no_offenses_full(cop: &dyn Cop, source_bytes: &[u8]) {
    assert_cop_no_offenses_full_with_config(cop, source_bytes, CopConfig::default());
}

pub fn assert_cop_no_offenses_full_with_config(
    cop: &dyn Cop,
    source_bytes: &[u8],
    config: CopConfig,
) {
    let parsed = parse_fixture(source_bytes);
    let filename = parsed.filename.as_deref().unwrap_or("test.rb");
    let diagnostics = run_cop_full_internal(cop, &parsed.source, config, filename);
    assert!(
        diagnostics.is_empty(),
        "Expected no offenses but got {}:\n{}",
        diagnostics.len(),
        format_diagnostics(&diagnostics),
    );
}
