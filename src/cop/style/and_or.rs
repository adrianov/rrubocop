//! Style/AndOr — ban `and`/`or` keywords.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct AndOr;

impl Cop for AndOr {
    fn name(&self) -> &'static str {
        "Style/AndOr"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["binary"]
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
            let repl = match text {
                b"and" => "&&",
                b"or" => "||",
                _ => continue,
            };
            let (line, col) = source.offset_to_line_col(child.start_byte());
            let mut diag = self.diagnostic(
                source,
                line,
                col,
                format!("Use `{repl}` instead of `{}`.", std::str::from_utf8(text).unwrap_or("")),
            );
            if let Some(ref mut corr) = corrections {
                corr.push(crate::correction::Correction {
                    start: child.start_byte(),
                    end: child.end_byte(),
                    replacement: repl.to_string(),
                    cop_name: self.name(),
                    cop_index: 0,
                });
                diag.corrected = true;
            }
            diagnostics.push(diag);
        }
    }
}
