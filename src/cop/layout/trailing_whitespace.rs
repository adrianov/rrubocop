//! Layout/TrailingWhitespace — adapted from nitrocop (line-based).

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct TrailingWhitespace;

fn trailing_whitespace_start(line: &[u8]) -> Option<usize> {
    let mut end = line.len();
    let mut found = false;
    while end > 0 {
        if matches!(line[end - 1], b' ' | b'\t') {
            end -= 1;
            found = true;
            continue;
        }
        if end >= 3 && line[end - 3..end] == [0xE3, 0x80, 0x80] {
            end -= 3;
            found = true;
            continue;
        }
        if end >= 2 && line[end - 2..end] == [0xC2, 0xA0] {
            end -= 2;
            found = true;
            continue;
        }
        break;
    }
    found.then_some(end)
}

fn strip_cr(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\r").unwrap_or(line)
}

fn heredoc_opener(line: &[u8]) -> Option<Vec<u8>> {
    let s = std::str::from_utf8(line).ok()?;
    let idx = s.find("<<")?;
    // Avoid shift: identifier/)/]/ before <<
    if idx > 0 {
        let before = &s.as_bytes()[..idx];
        if let Some(&b) = before.iter().rev().find(|&&b| b != b' ' && b != b'\t')
            && matches!(
                b,
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b')' | b']' | b'}' | b'@' | b'$'
            )
        {
            return None;
        }
    }
    let rest = &s[idx + 2..];
    let rest = rest.strip_prefix('~').or_else(|| rest.strip_prefix('-')).unwrap_or(rest);
    let rest = rest.trim_start();
    if rest.is_empty() || rest.as_bytes()[0].is_ascii_digit() {
        return None;
    }
    let ident: String = rest
        .chars()
        .skip_while(|c| *c == '\'' || *c == '"' || *c == '`')
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if ident.is_empty() {
        None
    } else {
        Some(ident.into_bytes())
    }
}

impl Cop for TrailingWhitespace {
    fn name(&self) -> &'static str {
        "Layout/TrailingWhitespace"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let allow_in_heredoc = config.get_bool("AllowInHeredoc", false);
        let lines: Vec<&[u8]> = source.lines().collect();
        let mut heredoc_terminators: Vec<Vec<u8>> = Vec::new();
        let mut saw_nonblank = false;
        let mut byte_offset = 0usize;

        for (i, line) in lines.iter().enumerate() {
            let stripped = strip_cr(line);

            if let Some(term) = heredoc_terminators.last() {
                let trimmed: Vec<u8> = stripped
                    .iter()
                    .copied()
                    .skip_while(|&b| b == b' ' || b == b'\t')
                    .collect();
                if &trimmed == term {
                    heredoc_terminators.pop();
                } else if allow_in_heredoc {
                    byte_offset += line.len() + 1;
                    continue;
                }
            }

            if stripped == b"__END__" && heredoc_terminators.is_empty() && saw_nonblank {
                break;
            }

            if !stripped.iter().all(|&b| b == b' ' || b == b'\t') {
                saw_nonblank = true;
            }

            if let Some(start) = trailing_whitespace_start(stripped) {
                let col = {
                    let prefix = &stripped[..start];
                    prefix.iter().filter(|&&b| (b & 0xC0) != 0x80).count()
                };
                let mut diag = self.diagnostic(
                    source,
                    i + 1,
                    col,
                    "Trailing whitespace detected.".to_string(),
                );
                if let Some(ref mut corr) = corrections {
                    let abs_start = byte_offset + start;
                    let abs_end = byte_offset + stripped.len();
                    corr.push(crate::correction::Correction {
                        start: abs_start,
                        end: abs_end,
                        replacement: String::new(),
                        cop_name: self.name(),
                        cop_index: 0,
                    });
                    diag.corrected = true;
                }
                diagnostics.push(diag);
            }

            if let Some(term) = heredoc_opener(stripped) {
                heredoc_terminators.push(term);
            }

            byte_offset += line.len() + 1;
        }
    }
}
