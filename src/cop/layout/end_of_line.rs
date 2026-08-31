//! Layout/EndOfLine — simplified port from nitrocop (line-based).

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EndOfLine;

impl Cop for EndOfLine {
    fn name(&self) -> &'static str {
        "Layout/EndOfLine"
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
        let style = config.get_str("EnforcedStyle", "native");
        let want_crlf = style == "crlf"
            || (style == "native" && cfg!(windows));

        match want_crlf {
            false => {
                let mut byte_offset = 0usize;
                for (i, line) in source.lines().enumerate() {
                    if line.ends_with(b"\r") {
                        let cr_offset = byte_offset + line.len() - 1;
                        let mut diag = self.diagnostic(
                            source,
                            i + 1,
                            line.len() - 1,
                            "Carriage return character detected.".to_string(),
                        );
                        if let Some(ref mut corr) = corrections {
                            corr.push(crate::correction::Correction {
                                start: cr_offset,
                                end: cr_offset + 1,
                                replacement: String::new(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                            diag.corrected = true;
                        }
                        diagnostics.push(diag);
                        break;
                    }
                    byte_offset += line.len() + 1;
                }
            }
            true => {
                let lines: Vec<&[u8]> = source.lines().collect();
                let mut byte_offset = 0usize;
                for (i, line) in lines.iter().enumerate() {
                    if i == lines.len() - 1 && line.is_empty() {
                        break;
                    }
                    // Last line without trailing newline: RuboCop skips
                    if i == lines.len() - 1 && !source.as_bytes().ends_with(b"\n") {
                        break;
                    }
                    if !line.ends_with(b"\r") {
                        let newline_offset = byte_offset + line.len();
                        let mut diag = self.diagnostic(
                            source,
                            i + 1,
                            line.len(),
                            "Carriage return character missing.".to_string(),
                        );
                        if let Some(ref mut corr) = corrections {
                            corr.push(crate::correction::Correction {
                                start: newline_offset,
                                end: newline_offset,
                                replacement: "\r".to_string(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                            diag.corrected = true;
                        }
                        diagnostics.push(diag);
                        break;
                    }
                    byte_offset += line.len() + 1;
                }
            }
        }
    }
}
