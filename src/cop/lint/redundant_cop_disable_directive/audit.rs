use std::collections::HashSet;

use super::directives::{collect_directives, DisableDirective};
use super::RedundantCopDisableDirective;
use crate::cop::{cop_ran_in_lint, Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

fn cop_matches_disabled(offense_cop: &str, disabled: &str) -> bool {
    if disabled.eq_ignore_ascii_case("all") {
        return true;
    }
    if disabled.eq_ignore_ascii_case(offense_cop) {
        return true;
    }
    offense_cop
        .split_once('/')
        .is_some_and(|(dept, _)| disabled.eq_ignore_ascii_case(dept))
}

fn offense_in_range(offense: &Diagnostic, range: (usize, usize)) -> bool {
    offense.location.line >= range.0 && offense.location.line <= range.1
}

fn directive_needed(cop: &str, range: (usize, usize), offenses: &[Diagnostic]) -> bool {
    offenses.iter().any(|o| {
        cop_matches_disabled(&o.cop_name, cop) && offense_in_range(o, range)
    })
}

fn active_for_redundant_audit(c: &dyn Cop) -> bool {
    cop_ran_in_lint(c) && c.redundant_disable_audit()
}

fn cop_in_department(c: &dyn Cop, dept: &str) -> bool {
    c.name()
        .split_once('/')
        .is_some_and(|(d, _)| d.eq_ignore_ascii_case(dept))
}

fn any_active_cop(active: &[(&dyn Cop, &CopConfig)], pred: impl Fn(&dyn Cop) -> bool) -> bool {
    active.iter().any(|(c, _)| pred(*c))
}

fn cop_auditable(name: &str, active: &[(&dyn Cop, &CopConfig)]) -> bool {
    if name.eq_ignore_ascii_case("all") {
        return any_active_cop(active, active_for_redundant_audit);
    }
    if name.contains('/') {
        return any_active_cop(active, |c| {
            c.name().eq_ignore_ascii_case(name) && active_for_redundant_audit(c)
        });
    }
    any_active_cop(active, |c| active_for_redundant_audit(c) && cop_in_department(c, name))
}

fn cop_highlight(line: &str, col: usize, cop: &str) -> usize {
    line[col..]
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .find(cop)
        .map(|_| cop.len())
        .unwrap_or(1)
}

fn report_redundant_disable(
    cop: &RedundantCopDisableDirective,
    source: &SourceFile,
    line: &str,
    dir: &DisableDirective,
    name: &str,
    offenses: &[Diagnostic],
    active: &[(&dyn Cop, &CopConfig)],
    seen: &mut HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !cop_auditable(name, active)
        || !seen.insert(name.to_string())
        || directive_needed(name, dir.range, offenses)
    {
        return;
    }
    let col = line
        .to_ascii_lowercase()
        .find(&name.to_ascii_lowercase())
        .unwrap_or(dir.column);
    let mut diag = cop.diagnostic(
        source,
        dir.line,
        col,
        format!("Unnecessary disabling of `{name}`."),
    );
    diag.highlight_length = cop_highlight(line, col, name);
    diagnostics.push(diag);
}

/// Post-pass: compare disable directives against offenses found before filtering.
pub fn audit_redundant_disables(
    cop: &RedundantCopDisableDirective,
    source: &SourceFile,
    offenses: &[Diagnostic],
    active: &[(&dyn Cop, &CopConfig)],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for dir in collect_directives(source) {
        let line = source.line_text(dir.line).unwrap_or("").to_string();
        let mut seen = HashSet::new();
        for name in &dir.cops {
            report_redundant_disable(
                cop,
                source,
                &line,
                &dir,
                name,
                offenses,
                active,
                &mut seen,
                diagnostics,
            );
        }
    }
}
