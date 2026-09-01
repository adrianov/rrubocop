//! Read RuboCop YAML; expand ERB when present.
//!
//! Plain YAML needs no Ruby. Simple ERB — literals, `<% var = ... %>` /
//! `<%= var %>`, and `ENV[...]` / `ENV.fetch(...)` — is expanded natively in Rust,
//! so common Shopify-style configs work without a Ruby runtime. Anything the native
//! pass does not fully understand falls back to `ruby` / `bundle exec ruby`, matching
//! RuboCop's ConfigLoader.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;

use super::gem_path;

const ERB_SCRIPT: &str = concat!(
    "path = ARGV[0]; ",
    "Dir.chdir(File.dirname(path)) { print ERB.new(File.read(path)).result }"
);

/// Read a config file: expand ERB when present, strip `!ruby/regexp`.
pub(crate) fn load_yaml_text(config_path: &Path, working_dir: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read config {}", config_path.display()))?;
    let expanded = if raw.contains("<%") {
        match expand_erb_native(&raw) {
            Some(text) => text,
            None => expand_erb(config_path, working_dir).with_context(|| {
                format!(
                    "ERB expansion failed for {} (need `bundle exec ruby` in {})",
                    config_path.display(),
                    working_dir.display()
                )
            })?,
        }
    } else {
        raw
    };
    Ok(expanded.replace("!ruby/regexp ", ""))
}

fn expand_erb(config_path: &Path, working_dir: &Path) -> Result<String> {
    let output = erb_command(working_dir)
        .arg(config_path)
        .output()
        .context("failed to run bundle exec ruby -rerb")?;
    if !output.status.success() {
        anyhow::bail!(
            "ruby ERB exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn erb_command(working_dir: &Path) -> Command {
    let has_gemfile = working_dir.join("Gemfile").exists();
    let mut cmd = if has_gemfile && gem_path::needs_mise_exec(working_dir) {
        let mut c = Command::new("mise");
        c.args(["exec", "--", "bundle", "exec", "ruby", "-rerb", "-e", ERB_SCRIPT]);
        c
    } else if has_gemfile {
        let mut c = Command::new("bundle");
        c.args(["exec", "ruby", "-rerb", "-e", ERB_SCRIPT]);
        c
    } else {
        let mut c = Command::new("ruby");
        c.args(["-rerb", "-e", ERB_SCRIPT]);
        c
    };
    cmd.current_dir(working_dir);
    cmd
}

enum Tag {
    Stmt,
    Expr,
    Comment,
}

/// Expand simple ERB without Ruby. Returns `None` for anything unsupported so the
/// caller falls back to Ruby, keeping byte-for-byte parity with RuboCop.
fn expand_erb_native(raw: &str) -> Option<String> {
    let mut out = String::with_capacity(raw.len());
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut rest = raw;
    while let Some(open) = rest.find("<%") {
        out.push_str(&rest[..open]);
        let after = &rest[open + 2..];
        if after.starts_with('%') {
            return None; // `<%%` literal escape — let Ruby handle it
        }
        let (kind, body) = match after.bytes().next()? {
            b'=' => (Tag::Expr, &after[1..]),
            b'#' => (Tag::Comment, &after[1..]),
            _ => (Tag::Stmt, after),
        };
        let close = body.find("%>")?;
        let inner = &body[..close];
        match kind {
            Tag::Comment => {}
            Tag::Expr => out.push_str(&eval_expr(inner, &vars)?),
            Tag::Stmt => apply_stmt(inner, &mut vars)?,
        }
        rest = &body[close + 2..];
    }
    out.push_str(rest);
    Some(out)
}

/// Run `<% ... %>` statements: only plain `name = <expr>` assignments are supported.
fn apply_stmt(stmt: &str, vars: &mut HashMap<String, String>) -> Option<()> {
    for part in stmt.split([';', '\n']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let eq = part.find('=')?;
        let name = part[..eq].trim();
        let rhs = part[eq + 1..].trim();
        if rhs.starts_with('=') || !is_ident(name) {
            return None; // comparison (`==`) or non-assignment — bail to Ruby
        }
        let value = eval_expr(rhs, vars)?;
        vars.insert(name.to_string(), value);
    }
    Some(())
}

/// Evaluate a `<%= ... %>` expression to the string Ruby's `#to_s` would produce.
fn eval_expr(expr: &str, vars: &HashMap<String, String>) -> Option<String> {
    let s = expr.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(v) = string_literal(s) {
        return Some(v);
    }
    match s {
        "true" => return Some("true".to_string()),
        "false" => return Some("false".to_string()),
        "nil" => return Some(String::new()),
        _ => {}
    }
    if let Some(v) = integer_literal(s) {
        return Some(v);
    }
    if s.starts_with("ENV[") || s.starts_with("ENV.fetch") {
        return env_expr(s, vars);
    }
    vars.get(s).cloned()
}

/// `ENV['NAME']` (nil → empty) and `ENV.fetch('NAME'[, default])`.
fn env_expr(expr: &str, vars: &HashMap<String, String>) -> Option<String> {
    if let Some(caps) = env_index_re().captures(expr) {
        return Some(std::env::var(&caps[1]).unwrap_or_default());
    }
    if let Some(caps) = env_fetch_re().captures(expr) {
        if let Ok(val) = std::env::var(&caps[1]) {
            return Some(val);
        }
        return caps.get(2).and_then(|d| eval_expr(d.as_str(), vars));
    }
    None
}

fn env_index_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"^ENV\[\s*['"]([^'"]+)['"]\s*\]$"#).unwrap())
}

fn env_fetch_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"^ENV\.fetch\(\s*['"]([^'"]+)['"]\s*(?:,\s*(.+?)\s*)?\)$"#).unwrap())
}

/// A single- or double-quoted string with no escapes or interpolation.
fn string_literal(s: &str) -> Option<String> {
    let quote = s.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let inner = s.strip_prefix(quote)?.strip_suffix(quote)?;
    if inner.contains(quote) || inner.contains('\\') {
        return None;
    }
    if quote == '"' && inner.contains("#{") {
        return None;
    }
    Some(inner.to_string())
}

/// A base-10 integer literal (optional `-`, `_` separators), matching Ruby `#to_s`.
fn integer_literal(s: &str) -> Option<String> {
    let digits = s.strip_prefix('-').unwrap_or(s);
    if !digits.starts_with(|c: char| c.is_ascii_digit())
        || digits.ends_with('_')
        || digits.contains("__")
        || !digits.bytes().all(|b| b.is_ascii_digit() || b == b'_')
    {
        return None;
    }
    let cleaned: String = digits.chars().filter(|c| *c != '_').collect();
    if cleaned.len() > 1 && cleaned.starts_with('0') {
        return None; // leading zero is an octal literal in Ruby — let Ruby decide
    }
    Some(if s.starts_with('-') {
        format!("-{cleaned}")
    } else {
        cleaned
    })
}

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn fixture_erb() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/config/erb/with_erb.yml")
    }

    fn abc_enabled(text: &str) -> Option<bool> {
        let raw: serde_yml::Value = serde_yml::from_str(text).ok()?;
        raw.get("Metrics/AbcSize")?.get("Enabled")?.as_bool()
    }

    /// Expand `raw` with the real Ruby ERB path; `None` when Ruby is unavailable.
    fn ruby_expand(raw: &str) -> Option<String> {
        Command::new("ruby").arg("-e").arg("").output().ok()?;
        let dir = tempfile::tempdir().ok()?;
        let path = dir.path().join("config.yml");
        std::fs::File::create(&path).ok()?.write_all(raw.as_bytes()).ok()?;
        let out = erb_command(dir.path()).arg(&path).output().ok()?;
        out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    }

    #[test]
    fn erb_expands_and_parses() {
        let path = fixture_erb();
        let text = load_yaml_text(&path, path.parent().unwrap()).unwrap();
        assert!(!text.contains("<%"));
        assert_eq!(abc_enabled(&text), Some(false));
    }

    #[test]
    fn native_expands_fixture_without_ruby() {
        let raw = std::fs::read_to_string(fixture_erb()).unwrap();
        let text = expand_erb_native(&raw).expect("simple ERB should expand natively");
        assert!(!text.contains("<%"));
        assert_eq!(abc_enabled(&text), Some(false));
    }

    #[test]
    fn native_handles_supported_forms() {
        unsafe { std::env::set_var("RRUBOCOP_MAX", "123") };
        let raw = concat!(
            "<% style = \"double_quotes\" %>\n",
            "Style/StringLiterals:\n",
            "  EnforcedStyle: <%= style %>\n",
            "Layout/LineLength:\n",
            "  Max: <%= ENV.fetch('RRUBOCOP_MAX', 80) %>\n",
            "Naming/Foo:\n",
            "  Enabled: <%= false %>\n",
            "  Default: <%= ENV.fetch('RRUBOCOP_UNSET', 'yes') %>\n",
        );
        let out = expand_erb_native(raw).expect("supported forms expand natively");
        assert!(out.contains("EnforcedStyle: double_quotes"), "{out}");
        assert!(out.contains("Max: 123"), "{out}");
        assert!(out.contains("Enabled: false"), "{out}");
        assert!(out.contains("Default: yes"), "{out}");
    }

    #[test]
    fn native_bails_on_complex() {
        for raw in [
            "<%= 1 + 1 %>",
            "<%= RUBY_VERSION %>",
            "<%= Dir.pwd %>",
            "<% if true %>\nx\n<% end %>",
            "<%% literal %>",
            "<%= 2.7 %>",
        ] {
            assert!(expand_erb_native(raw).is_none(), "should fall back: {raw:?}");
        }
    }

    #[test]
    fn native_matches_ruby() {
        unsafe { std::env::set_var("RRUBOCOP_PARITY", "77") };
        let cases = [
            "<% v = 42 %>\nMax: <%= v %>\n",
            "Max: <%= 1_000 %>\n",
            "S: <%= 'hi' %>\n",
            "S: <%= \"bye\" %>\n",
            "Enabled: <%= true %>\n",
            "Enabled: <%= false %>\n",
            "N: <%= nil %>|end\n",
            "Max: <%= ENV.fetch('RRUBOCOP_PARITY', 80) %>\n",
            "Def: <%= ENV.fetch('RRUBOCOP_UNSET', 'fallback') %>\n",
            "Get: <%= ENV['RRUBOCOP_PARITY'] %>\n",
            "Get: <%= ENV['RRUBOCOP_UNSET'] %>|end\n",
            "<% a = 1; b = 2 %>\n<%= a %>-<%= b %>\n",
            "<%# a comment %>\nStyle/Foo:\n  Enabled: true\n",
        ];
        for raw in cases {
            let native = expand_erb_native(raw).unwrap_or_else(|| panic!("native failed: {raw:?}"));
            if let Some(ruby) = ruby_expand(raw) {
                assert_eq!(native, ruby, "mismatch for {raw:?}");
            }
        }
    }
}
