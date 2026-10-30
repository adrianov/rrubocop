//! Layout/LineLength — lines must not exceed Max columns.

use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::{Node, Tree};

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct LineLength;

static QUALIFIED_NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:[A-Z][A-Za-z0-9_]*::)+[A-Za-z_][A-Za-z0-9_]*\b")
        .expect("qualified name regex")
});

struct Settings {
    max: usize,
    allow_hdoc: bool,
    allow_uri: bool,
    allow_qualified: bool,
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

fn load_settings(config: &CopConfig, tree: &Tree) -> Settings {
    let mut schemes = string_list(config, "URISchemes");
    if schemes.is_empty() {
        schemes = vec!["http".into(), "https".into()];
    }
    let allow_hdoc = config.get_bool("AllowHeredoc", true);
    Settings {
        max: config.get_usize("Max", 120),
        allow_hdoc,
        allow_uri: config.get_bool("AllowURI", true),
        allow_qualified: config.get_bool("AllowQualifiedName", true),
        ignore_directives: config.get_bool("IgnoreCopDirectives", true),
        schemes,
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

/// RuboCop `check_directive_line`: code before a `# rubocop:` directive, trailing
/// space trimmed. Such lines are measured without the directive and get no exemptions.
fn before_directive<'a>(cfg: &Settings, text: &'a str) -> Option<&'a str> {
    if !cfg.ignore_directives {
        return None;
    }
    let at = text.find("# rubocop:").or_else(|| text.find("#rubocop:"))?;
    Some(text[..at].trim_end())
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

/// Character column of byte offset `at` (RuboCop compares columns, not bytes).
fn char_col(text: &str, at: usize) -> usize {
    text[..at].chars().count()
}

fn uri_allows(cfg: &Settings, text: &str) -> bool {
    cfg.allow_uri && uri_start(text, &cfg.schemes).is_some_and(|at| char_col(text, at) < cfg.max)
}

/// RuboCop `extend_end_position`: a line with `{` that ends in `}` (YARD
/// `{<uri> <title>}`) extends to EOL; otherwise absorb trailing non-space (e.g. `)`).
fn extend_end(line: &str, end: usize) -> usize {
    if line
        .strip_suffix('}')
        .is_some_and(|head| head.contains('{'))
    {
        return line.len();
    }
    end + line[end..]
        .char_indices()
        .take_while(|(_, c)| !c.is_whitespace())
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0)
}

/// RuboCop AllowQualifiedName: last `Foo::Bar` starts before Max and reaches EOL.
fn qualified_allows(cfg: &Settings, text: &str) -> bool {
    if !cfg.allow_qualified {
        return false;
    }
    let Some(m) = QUALIFIED_NAME_RE.find_iter(text).last() else {
        return false;
    };
    char_col(text, m.start()) < cfg.max && extend_end(text, m.end()) == text.len()
}

/// AllowedPatterns, or (unless a directive is on the line) a URI / qualified-name tail.
fn allowed_long(cfg: &Settings, text: &str, directive: bool) -> bool {
    cfg.patterns.iter().any(|re| re.is_match(text))
        || (!directive && (uri_allows(cfg, text) || qualified_allows(cfg, text)))
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
    let text = String::from_utf8_lossy(raw);
    let code = before_directive(cfg, &text);
    let len = code.unwrap_or(&text).chars().count();
    if len <= cfg.max || allowed_long(cfg, &text, code.is_some()) {
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
            let raw = raw.strip_suffix(b"\r").unwrap_or(raw);
            check_line(self, source, &cfg, idx, raw, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(LineLength, "cops/layout/line_length");

    #[test]
    fn crlf_not_counted() {
        let line = format!("x = '{}'\r\n", "a".repeat(114));
        crate::testutil::assert_cop_no_offenses_full(&LineLength, line.as_bytes());
    }
}
