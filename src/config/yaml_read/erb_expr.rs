//! Expression evaluation for native ERB (literals / vars / ENV).

use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;

pub(super) fn eval_expr(expr: &str, vars: &HashMap<String, String>) -> Option<String> {
    let s = expr.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(v) = string_literal(s) {
        return Some(v);
    }
    if let Some(v) = bool_or_nil(s) {
        return Some(v);
    }
    if let Some(v) = integer_literal(s) {
        return Some(v);
    }
    if s.starts_with("ENV[") || s.starts_with("ENV.fetch") {
        return env_expr(s, vars);
    }
    vars.get(s).cloned()
}

fn bool_or_nil(s: &str) -> Option<String> {
    match s {
        "true" => Some("true".to_string()),
        "false" => Some("false".to_string()),
        "nil" => Some(String::new()),
        _ => None,
    }
}

fn env_expr(expr: &str, vars: &HashMap<String, String>) -> Option<String> {
    if let Some(caps) = env_index_re().captures(expr) {
        return Some(std::env::var(&caps[1]).unwrap_or_default());
    }
    env_fetch(expr, vars)
}

fn env_fetch(expr: &str, vars: &HashMap<String, String>) -> Option<String> {
    let caps = env_fetch_re().captures(expr)?;
    if let Ok(val) = std::env::var(&caps[1]) {
        return Some(val);
    }
    caps.get(2).and_then(|d| eval_expr(d.as_str(), vars))
}

fn env_index_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"^ENV\[\s*['"]([^'"]+)['"]\s*\]$"#).unwrap())
}

fn env_fetch_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"^ENV\.fetch\(\s*['"]([^'"]+)['"]\s*(?:,\s*(.+?)\s*)?\)$"#).unwrap()
    })
}

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

fn integer_literal(s: &str) -> Option<String> {
    let digits = s.strip_prefix('-').unwrap_or(s);
    if !valid_int_digits(digits) {
        return None;
    }
    let cleaned: String = digits.chars().filter(|c| *c != '_').collect();
    if cleaned.len() > 1 && cleaned.starts_with('0') {
        return None;
    }
    Some(if s.starts_with('-') {
        format!("-{cleaned}")
    } else {
        cleaned
    })
}

fn valid_int_digits(digits: &str) -> bool {
    digits.starts_with(|c: char| c.is_ascii_digit())
        && !digits.ends_with('_')
        && !digits.contains("__")
        && digits.bytes().all(|b| b.is_ascii_digit() || b == b'_')
}

pub(super) fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}
