//! Style/FrozenStringLiteralComment.

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FrozenStringLiteralComment;

fn has_frozen_comment(source: &SourceFile) -> bool {
    for line in source.lines().take(3) {
        let Ok(s) = std::str::from_utf8(line) else {
            continue;
        };
        let t = s.trim();
        if t.starts_with("#")
            && t.contains("frozen_string_literal:")
            && (t.contains("true") || t.contains("false"))
        {
            return true;
        }
        if t.starts_with("#!") {
            continue;
        }
        if !t.is_empty() && !t.starts_with('#') {
            break;
        }
    }
    false
}

impl Cop for FrozenStringLiteralComment {
    fn name(&self) -> &'static str {
        "Style/FrozenStringLiteralComment"
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
        let style = config.get_str("EnforcedStyle", "always");
        if style == "never" {
            return;
        }
        if source.as_bytes().is_empty() {
            return;
        }
        if has_frozen_comment(source) {
            return;
        }
        let mut diag = self.diagnostic(
            source,
            1,
            0,
            "Missing frozen string literal comment.".to_string(),
        );
        if let Some(ref mut corr) = corrections {
            let insert = b"# frozen_string_literal: true\n";
            let mut start = 0usize;
            if source.as_bytes().starts_with(b"#!") {
                if let Some(nl) = source.as_bytes().iter().position(|&b| b == b'\n') {
                    start = nl + 1;
                }
            }
            corr.push(crate::correction::Correction {
                start,
                end: start,
                replacement: String::from_utf8_lossy(insert).into_owned(),
                cop_name: self.name(),
                cop_index: 0,
            });
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}
