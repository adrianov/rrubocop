//! Layout/TrailingEmptyLines — adapted from nitrocop.

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct TrailingEmptyLines;

fn contains_end_marker(bytes: &[u8]) -> bool {
    bytes.windows(7).any(|w| w == b"__END__")
}

fn trailing_whitespace_info(bytes: &[u8]) -> (usize, usize) {
    let mut ws_len = 0;
    let mut newline_count = 0;
    for &b in bytes.iter().rev() {
        if matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0b' | b'\x0c') {
            ws_len += 1;
            if b == b'\n' {
                newline_count += 1;
            }
        } else {
            break;
        }
    }
    (ws_len, newline_count)
}

fn first_trailing_blank_line_offset(bytes: &[u8], trailing_start: usize) -> usize {
    let mut offset = trailing_start;
    if bytes.get(offset) == Some(&b'\r') {
        offset += 1;
    }
    if bytes.get(offset) == Some(&b'\n') {
        offset += 1;
    }
    offset.min(bytes.len().saturating_sub(1))
}

fn missing_blank_line_report_location(
    source: &SourceFile,
    bytes: &[u8],
    trailing_start: usize,
) -> (usize, usize) {
    if matches!(bytes.get(trailing_start), Some(b'\n' | b'\r')) {
        let (last_line, _) = source.offset_to_line_col(trailing_start);
        (last_line + 1, 0)
    } else {
        source.offset_to_line_col(trailing_start + 1)
    }
}

impl Cop for TrailingEmptyLines {
    fn name(&self) -> &'static str {
        "Layout/TrailingEmptyLines"
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
        let style = config.get_str("EnforcedStyle", "final_newline");
        let bytes = source.as_bytes();
        if bytes.is_empty() || contains_end_marker(bytes) || bytes.ends_with(b"%\n\n") {
            return;
        }

        let (ws_len, newline_count) = trailing_whitespace_info(bytes);
        let blank_lines = newline_count as isize - 1;
        let wanted_blank_lines: isize = if style == "final_blank_line" { 1 } else { 0 };
        if blank_lines == wanted_blank_lines {
            return;
        }

        let message = match blank_lines {
            -1 => "Final newline missing.".to_string(),
            0 => "Trailing blank line missing.".to_string(),
            1 => "Trailing blank line detected.".to_string(),
            n => {
                if wanted_blank_lines == 0 {
                    format!("{n} trailing blank lines detected.")
                } else {
                    format!("{n} trailing blank lines instead of {wanted_blank_lines} detected.")
                }
            }
        };

        let begin_pos = bytes.len() - ws_len;
        let (report_line, report_col) = if blank_lines == 0 {
            missing_blank_line_report_location(source, bytes, begin_pos)
        } else if blank_lines > 0 {
            source.offset_to_line_col(first_trailing_blank_line_offset(bytes, begin_pos))
        } else if ws_len > 0 {
            source.offset_to_line_col(begin_pos + 1)
        } else {
            source.offset_to_line_col(begin_pos)
        };

        let mut diag = self.diagnostic(source, report_line, report_col, message);
        if let Some(ref mut corr) = corrections {
            let replacement = if style == "final_blank_line" {
                "\n\n".to_string()
            } else {
                "\n".to_string()
            };
            corr.push(crate::correction::Correction {
                start: begin_pos,
                end: bytes.len(),
                replacement,
                cop_name: self.name(),
                cop_index: 0,
            });
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}
