//! Gemspec/DuplicatedAssignment — flag repeated gemspec attribute writes.

use std::collections::HashMap;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct DuplicatedAssignment;

impl Cop for DuplicatedAssignment {
    fn name(&self) -> &'static str {
        "Gemspec/DuplicatedAssignment"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*.gemspec"]
    }

    fn uses_line_phase(&self) -> bool {
        true
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut seen: HashMap<String, usize> = HashMap::new();
        for (line_idx, line) in source.lines().enumerate() {
            let Ok(line_str) = std::str::from_utf8(line) else {
                continue;
            };
            let trimmed = line_str.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            record_attrs(self, source, line_str, line_idx + 1, trimmed, &mut seen, diagnostics);
        }
    }
}

fn record_attrs(
    cop: &DuplicatedAssignment,
    source: &SourceFile,
    line_str: &str,
    line_num: usize,
    trimmed: &str,
    seen: &mut HashMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let col = line_str.find('.').unwrap_or(0) + 1;
    for attr in assignment_attrs(trimmed) {
        if let Some(&first) = seen.get(&attr) {
            diagnostics.push(cop.diagnostic(
                source,
                line_num,
                col,
                format!("Attribute `{attr}` is already set on line {first}."),
            ));
        } else {
            seen.insert(attr, line_num);
        }
    }
}

/// Attribute names from `spec.name = 'foo'` (skips `<<` / `==`; supports self-assign chains).
fn assignment_attrs(trimmed: &str) -> Vec<String> {
    let mut attrs = Vec::new();
    let mut at = 0;
    while let Some((attr, next)) = next_assignment(trimmed, at) {
        attrs.push(attr);
        at = next;
    }
    attrs
}

fn next_assignment(trimmed: &str, search_from: usize) -> Option<(String, usize)> {
    let (rel_dot, attr_name, after_attr) = find_dot_attr(&trimmed[search_from..])?;
    let skip = search_from + rel_dot + 1 + attr_name.len();
    match take_assignment(attr_name, after_attr) {
        Some(full) => Some((full, after_eq(trimmed, search_from + rel_dot)?)),
        None => next_assignment(trimmed, skip),
    }
}

fn take_assignment(attr_name: &str, after_attr: &str) -> Option<String> {
    let (full, rest) = extend_bracket(attr_name, after_attr)?;
    is_assignment(rest).then_some(full)
}

fn after_eq(trimmed: &str, abs_dot: usize) -> Option<usize> {
    Some(abs_dot + trimmed[abs_dot..].find('=')? + 1)
}

fn find_dot_attr(remaining: &str) -> Option<(usize, &str, &str)> {
    let rel_dot = remaining.find('.')?;
    let after_dot = &remaining[rel_dot + 1..];
    let attr_end = after_dot
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(after_dot.len());
    (attr_end > 0)
        .then(|| (rel_dot, &after_dot[..attr_end], &after_dot[attr_end..]))
        .or_else(|| {
            find_dot_attr(&remaining[rel_dot + 1..]).map(|(r, a, b)| (rel_dot + 1 + r, a, b))
        })
}

fn extend_bracket<'a>(attr_name: &str, after_attr: &'a str) -> Option<(String, &'a str)> {
    if !after_attr.starts_with('[') {
        return Some((attr_name.to_string(), after_attr.trim_start()));
    }
    let end = after_attr.find(']')?;
    Some((
        format!("{attr_name}{}", &after_attr[..=end]),
        after_attr[end + 1..].trim_start(),
    ))
}

fn is_assignment(rest: &str) -> bool {
    let b = rest.as_bytes();
    matches!(b, [b'=', b' ', ..] | [b'=', b'\n', ..] | [b'='])
        || (b.first() == Some(&b'=') && b.get(1) != Some(&b'='))
}
