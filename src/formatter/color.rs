//! RuboCop/Rainbow-compatible ANSI colors for progress and text output.

use std::io::IsTerminal;

use crate::diagnostic::Severity;

/// When color is on: wrap with SGR; when off: return text unchanged.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    enabled: bool,
}

impl Color {
    /// RuboCop `options[:color]`: `Some(true/false)` forces; `None` follows TTY + `NO_COLOR`.
    pub fn resolve(force: Option<bool>) -> Self {
        let enabled = match force {
            Some(v) => v,
            None => std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
        };
        Self { enabled }
    }

    fn paint(self, text: &str, code: u8) -> String {
        if self.enabled {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    pub fn red(self, text: &str) -> String {
        self.paint(text, 31)
    }

    pub fn green(self, text: &str) -> String {
        self.paint(text, 32)
    }

    pub fn yellow(self, text: &str) -> String {
        self.paint(text, 33)
    }

    pub fn magenta(self, text: &str) -> String {
        self.paint(text, 35)
    }

    pub fn cyan(self, text: &str) -> String {
        self.paint(text, 36)
    }

    /// RuboCop `COLOR_FOR_SEVERITY` letter coloring.
    pub fn severity_letter(self, severity: Severity) -> String {
        let letter = severity.letter().to_string();
        match severity {
            Severity::Convention => self.yellow(&letter),
            Severity::Warning => self.magenta(&letter),
            Severity::Error | Severity::Fatal => self.red(&letter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_is_identity() {
        let c = Color { enabled: false };
        assert_eq!(c.red("x"), "x");
        assert_eq!(c.severity_letter(Severity::Convention), "C");
    }

    #[test]
    fn enabled_wraps_sgr() {
        let c = Color { enabled: true };
        assert_eq!(c.yellow("C"), "\x1b[33mC\x1b[0m");
        assert_eq!(c.cyan("p"), "\x1b[36mp\x1b[0m");
        assert_eq!(c.red("1 offense"), "\x1b[31m1 offense\x1b[0m");
        assert_eq!(c.green("."), "\x1b[32m.\x1b[0m");
    }
}
