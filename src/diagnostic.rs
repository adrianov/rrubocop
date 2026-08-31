//! Offense diagnostics matching RuboCop text/JSON shapes.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Convention,
    Warning,
    Error,
    Fatal,
}

impl Severity {
    pub fn letter(&self) -> char {
        match self {
            Severity::Convention => 'C',
            Severity::Warning => 'W',
            Severity::Error => 'E',
            Severity::Fatal => 'F',
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Severity> {
        match s.to_lowercase().as_str() {
            "convention" | "c" => Some(Severity::Convention),
            "warning" | "w" => Some(Severity::Warning),
            "error" | "e" => Some(Severity::Error),
            "fatal" | "f" => Some(Severity::Fatal),
            _ => None,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.letter())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    /// 1-indexed line number
    pub line: usize,
    /// 0-indexed column (character offset within the line)
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub path: String,
    pub location: Location,
    pub severity: Severity,
    pub cop_name: String,
    pub message: String,
    #[serde(default)]
    pub corrected: bool,
}

impl Diagnostic {
    pub fn sort_key(&self) -> (&str, usize, usize) {
        (&self.path, self.location.line, self.location.column)
    }

    /// Clang/progress offense line (RuboCop colors when `color` is enabled).
    pub fn render(&self, color: crate::formatter::color::Color) -> String {
        let path = smart_path(&self.path);
        let mut s = String::new();
        if self.corrected {
            s.push_str(&color.green("[Corrected] "));
        }
        s.push_str(&format!(
            "{}:{}:{}: {}: {}: {}",
            color.cyan(&path),
            self.location.line,
            self.location.column + 1,
            color.severity_letter(self.severity),
            self.cop_name,
            annotate_message(&self.message, color),
        ));
        s
    }
}

/// RuboCop PathUtil.smart_path: prefer cwd-relative, drop a leading `./`.
pub fn smart_path(path: &str) -> String {
    let path = path.strip_prefix("./").unwrap_or(path);
    let p = std::path::Path::new(path);
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(rel) = p.strip_prefix(&cwd) {
            let s = rel.to_string_lossy();
            if !s.is_empty() {
                return s.into_owned();
            }
        }
    }
    path.to_string()
}

/// RuboCop SimpleTextFormatter#annotate_message: strip `` `...` ``; yellow insides when colored.
fn annotate_message(msg: &str, color: crate::formatter::color::Color) -> String {
    let mut out = String::with_capacity(msg.len());
    let mut rest = msg;
    while let Some(start) = rest.find('`') {
        out.push_str(&rest[..start]);
        rest = &rest[start + 1..];
        match rest.find('`') {
            Some(end) => {
                out.push_str(&color.yellow(&rest[..end]));
                rest = &rest[end + 1..];
            }
            None => {
                out.push('`');
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Uncolored (tests, non-TTY sinks).
        write!(
            f,
            "{}",
            self.render(crate::formatter::color::Color::resolve(Some(false)))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smart_path_strips_dot_slash() {
        assert_eq!(smart_path("./lib/a.rb"), "lib/a.rb");
        assert_eq!(smart_path("lib/a.rb"), "lib/a.rb");
    }

    #[test]
    fn smart_path_relativizes_under_cwd() {
        let cwd = std::env::current_dir().unwrap();
        let abs = cwd.join("lib/a.rb");
        assert_eq!(smart_path(&abs.to_string_lossy()), "lib/a.rb");
        // Sibling prefix must not match (Path::strip_prefix, not string prefix).
        let sibling = format!("{}2/x.rb", cwd.display());
        assert_eq!(smart_path(&sibling), sibling);
    }

    #[test]
    fn display_strips_message_backticks() {
        let d = Diagnostic {
            path: "./lib/a.rb".into(),
            location: Location { line: 10, column: 2 },
            severity: Severity::Convention,
            cop_name: "Metrics/AbcSize".into(),
            message: "Assignment Branch Condition size for `foo` is too high. [<1, 2, 3> 3.74/17]"
                .into(),
            corrected: false,
        };
        let s = d.to_string();
        assert!(s.starts_with("lib/a.rb:10:3: C: Metrics/AbcSize: "));
        assert!(s.contains("for foo is too high"));
        assert!(!s.contains("`foo`"));
    }

    #[test]
    fn render_colors_path_severity_and_backticks() {
        use crate::formatter::color::Color;
        let d = Diagnostic {
            path: "lib/a.rb".into(),
            location: Location { line: 10, column: 2 },
            severity: Severity::Convention,
            cop_name: "Metrics/AbcSize".into(),
            message: "size for `foo` is too high".into(),
            corrected: false,
        };
        let s = d.render(Color::resolve(Some(true)));
        assert!(s.contains("\x1b[36mlib/a.rb\x1b[0m"));
        assert!(s.contains("\x1b[33mC\x1b[0m"));
        assert!(s.contains("\x1b[33mfoo\x1b[0m"));
        assert!(!s.contains("`foo`"));
    }
}
