//! Style/Not — prefer `!` over `not`.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct Not;

impl Cop for Not {
    fn name(&self) -> &'static str {
        "Style/Not"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["unary"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let text = &source.as_bytes()[child.start_byte()..child.end_byte()];
            if text != b"not" {
                continue;
            }
            let (line, col) = source.offset_to_line_col(child.start_byte());
            let mut diag = self.diagnostic(
                source,
                line,
                col,
                "Use `!` instead of `not`.".to_string(),
            );
            if let Some(ref mut corr) = corrections {
                corr.push(crate::correction::Correction {
                    start: child.start_byte(),
                    end: child.end_byte(),
                    replacement: "!".into(),
                    cop_name: self.name(),
                    cop_index: 0,
                });
                diag.corrected = true;
            }
            diagnostics.push(diag);
        }
    }
}
