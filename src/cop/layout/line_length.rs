//! Layout/LineLength — lines must not exceed Max columns.

use regex::Regex;
use tree_sitter::{Node, Tree};

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct LineLength;

struct Settings {
    max: usize,
    allow_hdoc: bool,
    allow_uri: bool,
    ignore_directives: bool,
    schemes: Vec<String>,
    patterns: Vec<Regex>,
    heredocs: Vec<(usize, usize)>,
}

fn string_list(config: &CopConfig, key: &str) -> Vec<String> {
    match config.options.get(key) {
        Some(serde_yml::Value::Sequence(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        Some(serde_yml::Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn allow_heredoc(config: &CopConfig) -> bool {
    !matches!(
        config.options.get("AllowHeredoc"),
        Some(serde_yml::Value::Bool(false))
    )
}

fn load_settings(config: &CopConfig, tree: &Tree) -> Settings {
    let schemes = string_list(config, "URISchemes");
    let allow_hdoc = allow_heredoc(config);
    Settings {
        max: config.get_usize("Max", 120),
        allow_hdoc,
        allow_uri: config.get_bool("AllowURI", true),
        ignore_directives: config.get_bool("IgnoreCopDirectives", true),
        schemes: if schemes.is_empty() {
            vec!["http".into(), "https".into()]
        } else {
            schemes
        },
        patterns: string_list(config, "AllowedPatterns")
            .into_iter()
            .filter_map(|p| Regex::new(&p).ok())
            .collect(),
        heredocs: if allow_hdoc {
            heredoc_body_lines(tree)
        } else {
            Vec::new()
        },
    }
}

fn display_len(line: &[u8]) -> usize {
    String::from_utf8_lossy(line).chars().count()
}

fn strip_directive(line: &[u8]) -> &[u8] {
    let s = std::str::from_utf8(line).unwrap_or("");
    let cut = s
        .find("# rubocop:")
        .or_else(|| s.find("#rubocop:"))
        .unwrap_or(s.len());
    let mut end = cut;
    while end > 0 && matches!(line[end - 1], b' ' | b'\t') {
        end -= 1;
    }
    &line[..end]
}

fn uri_start(line: &str, schemes: &[String]) -> Option<usize> {
    schemes
        .iter()
        .filter_map(|sch| line.find(&format!("{sch}://")))
        .min()
}

fn heredoc_body_lines(tree: &Tree) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    fn walk(n: Node<'_>, out: &mut Vec<(usize, usize)>) {
        if n.kind() == "heredoc_body" {
            out.push((n.start_position().row + 1, n.end_position().row + 1));
        }
        let mut c = n.walk();
        for ch in n.children(&mut c) {
            walk(ch, out);
        }
    }
    walk(tree.root_node(), &mut out);
    out
}

fn in_heredoc(ranges: &[(usize, usize)], line: usize) -> bool {
    ranges.iter().any(|&(a, b)| (a..=b).contains(&line))
}

fn patterns_match(line: &str, patterns: &[Regex]) -> bool {
    patterns.iter().any(|re| re.is_match(line))
}

fn uri_allows(cfg: &Settings, text: &str) -> bool {
    cfg.allow_uri
        && uri_start(text, &cfg.schemes).is_some_and(|at| at < cfg.max)
}

fn check_line(
    cop: &LineLength,
    source: &SourceFile,
    cfg: &Settings,
    idx: usize,
    raw: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let line_no = idx + 1;
    if (idx == 0 && raw.starts_with(b"#!"))
        || (cfg.allow_hdoc && in_heredoc(&cfg.heredocs, line_no))
    {
        return;
    }
    let line = if cfg.ignore_directives {
        strip_directive(raw)
    } else {
        raw
    };
    let text = String::from_utf8_lossy(line);
    let len = display_len(line);
    if patterns_match(&text, &cfg.patterns) || len <= cfg.max || uri_allows(cfg, &text) {
        return;
    }
    diagnostics.push(cop.diagnostic(
        source,
        line_no,
        cfg.max,
        format!("Line is too long. [{len}/{}]", cfg.max),
    ));
}

impl Cop for LineLength {
    fn name(&self) -> &'static str {
        "Layout/LineLength"
    }

    fn uses_source_phase(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        _code_map: &crate::parse::codemap::CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let cfg = load_settings(config, tree);
        for (idx, raw) in source.lines().enumerate() {
            check_line(self, source, &cfg, idx, raw, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(LineLength, "cops/layout/line_length");
}
