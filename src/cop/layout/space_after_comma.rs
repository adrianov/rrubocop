//! Layout/SpaceAfterComma — space after `,` (simplified).

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceAfterComma;

impl Cop for SpaceAfterComma {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterComma"
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
        let bytes = source.as_bytes();
        let mut i = 0usize;
        while i + 1 < bytes.len() {
            if bytes[i] == b',' {
                let next = bytes[i + 1];
                if next != b' ' && next != b'\n' && next != b'\r' && next != b']' && next != b')' && next != b'}' {
                    let (line, col) = source.offset_to_line_col(i + 1);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "Space missing after comma.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: i + 1,
                            end: i + 1,
                            replacement: " ".into(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }
            }
            i += 1;
        }
    }
}
