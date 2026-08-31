//! Layout/EmptyLines — empty lines around code (simplified: consecutive blanks > 1).

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLines;

impl Cop for EmptyLines {
    fn name(&self) -> &'static str {
        "Layout/EmptyLines"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let lines: Vec<&[u8]> = source.lines().collect();
        let mut byte_offset = 0usize;
        let mut blank_run_start: Option<(usize, usize)> = None; // (line_idx, byte)
        let mut blank_count = 0usize;

        for (i, line) in lines.iter().enumerate() {
            let is_blank = line.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r');
            if is_blank {
                if blank_count == 0 {
                    blank_run_start = Some((i, byte_offset));
                }
                blank_count += 1;
            } else {
                if blank_count > 1
                    && let Some((start_line, start_byte)) = blank_run_start
                {
                    let mut diag = self.diagnostic(
                        source,
                        start_line + 2,
                        0,
                        "Extra blank line detected.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        // Keep one blank: remove from after first blank newline to start of next blank's end
                        let keep_end = start_byte
                            + lines[start_line].len()
                            + 1; // first blank line + \n
                        let remove_end = byte_offset;
                        if remove_end > keep_end {
                            corr.push(crate::correction::Correction {
                                start: keep_end,
                                end: remove_end,
                                replacement: String::new(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                            diag.corrected = true;
                        }
                    }
                    diagnostics.push(diag);
                }
                blank_count = 0;
                blank_run_start = None;
            }
            byte_offset += line.len() + 1;
        }
    }
}
