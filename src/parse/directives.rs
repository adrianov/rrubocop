//! `# rubocop:disable` / `enable` directive parsing.

use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Directives {
    /// Cop name (or department or "all") → disabled line numbers (trailing).
    line_disables: HashMap<String, HashSet<usize>>,
    /// Cop name → inclusive ranges disabled by block directives.
    range_disables: HashMap<String, Vec<(usize, usize)>>,
    open_blocks: HashMap<String, usize>,
}

impl Directives {
    pub fn suppresses(&self, cop_name: &str, line: usize) -> bool {
        if self.line_disabled("all", line) || self.range_disabled("all", line) {
            return true;
        }
        if self.line_disabled(cop_name, line) || self.range_disabled(cop_name, line) {
            return true;
        }
        if let Some((dept, _)) = cop_name.split_once('/') {
            self.line_disabled(dept, line) || self.range_disabled(dept, line)
        } else {
            false
        }
    }

    fn line_disabled(&self, name: &str, line: usize) -> bool {
        self.line_disables
            .get(name)
            .is_some_and(|s| s.contains(&line))
    }

    fn range_disabled(&self, name: &str, line: usize) -> bool {
        self.range_disables
            .get(name)
            .is_some_and(|ranges| ranges.iter().any(|&(a, b)| a <= line && line <= b))
    }
}

fn apply_enable(d: &mut Directives, rest: &str, line_no: usize) {
    for name in cop_names(rest.trim_start_matches([' ', ':'])) {
        if let Some(start) = d.open_blocks.remove(&name) {
            d.range_disables
                .entry(name)
                .or_default()
                .push((start, line_no));
        }
    }
}

fn apply_disable(d: &mut Directives, rest: &str, line: &str, line_no: usize) {
    let mut names = cop_names(rest.trim_start_matches([' ', ':']));
    if names.is_empty() {
        names.push("all".into());
    }
    let trailing = !line.trim_start().starts_with('#');
    for name in names {
        if trailing {
            d.line_disables
                .entry(name)
                .or_default()
                .insert(line_no);
        } else {
            d.open_blocks.entry(name).or_insert(line_no);
        }
    }
}

fn apply_line(d: &mut Directives, line: &str, line_no: usize) {
    let Some(comment) = extract_rubocop_comment(line) else {
        return;
    };
    if let Some(rest) = comment.strip_prefix("enable") {
        apply_enable(d, rest, line_no);
        return;
    }
    if let Some(rest) = comment
        .strip_prefix("disable")
        .or_else(|| comment.strip_prefix("todo"))
    {
        apply_disable(d, rest, line, line_no);
    }
}

pub fn parse(src: &str) -> Directives {
    let mut d = Directives::default();
    let mut line_no = 0usize;
    for line in src.lines() {
        line_no += 1;
        apply_line(&mut d, line, line_no);
    }
    for (name, start) in d.open_blocks.drain() {
        d.range_disables
            .entry(name)
            .or_default()
            .push((start, line_no.max(1)));
    }
    d
}

fn extract_rubocop_comment(line: &str) -> Option<&str> {
    let idx = line.find('#')?;
    line[idx + 1..].trim_start().strip_prefix("rubocop:")
}

fn cop_names(after: &str) -> Vec<String> {
    let after = after.split("--").next().unwrap_or(after);
    after
        .split(',')
        .map(|s| s.trim().trim_start_matches(':').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
