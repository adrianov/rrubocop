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

pub fn parse(src: &str) -> Directives {
    let mut d = Directives::default();
    let mut line_no = 0usize;
    for line in src.lines() {
        line_no += 1;
        let Some(comment) = extract_rubocop_comment(line) else {
            continue;
        };
        if let Some(rest) = comment.strip_prefix("enable") {
            let names = cop_names(rest.trim_start_matches([' ', ':']));
            for name in names {
                if let Some(start) = d.open_blocks.remove(&name) {
                    d.range_disables
                        .entry(name)
                        .or_default()
                        .push((start, line_no));
                }
            }
            continue;
        }
        if let Some(rest) = comment
            .strip_prefix("disable")
            .or_else(|| comment.strip_prefix("todo"))
        {
            let names = cop_names(rest.trim_start_matches([' ', ':']));
            let names = if names.is_empty() {
                vec!["all".to_string()]
            } else {
                names
            };
            let trailing = !line.trim_start().starts_with('#');
            for name in names {
                if trailing {
                    d.line_disables
                        .entry(name.clone())
                        .or_default()
                        .insert(line_no);
                } else {
                    d.open_blocks.entry(name).or_insert(line_no);
                }
            }
        }
    }
    // Unclosed blocks disable through EOF
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
    let after = line[idx + 1..].trim_start();
    after.strip_prefix("rubocop:")
}

fn cop_names(after: &str) -> Vec<String> {
    let after = after.split("--").next().unwrap_or(after);
    after
        .split(',')
        .map(|s| s.trim().trim_start_matches(':').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
