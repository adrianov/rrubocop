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
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.corrected {
            write!(f, "[Corrected] ")?;
        }
        write!(
            f,
            "{}:{}:{}: {}: {}: {}",
            self.path,
            self.location.line,
            self.location.column + 1,
            self.severity,
            self.cop_name,
            self.message,
        )
    }
}
