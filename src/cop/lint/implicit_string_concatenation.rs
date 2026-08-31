use tree_sitter::Node;

use crate::cop::shared::{node_line, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/ImplicitStringConcatenation — adjacent string literals on the same line.
pub struct ImplicitStringConcatenation;

impl Cop for ImplicitStringConcatenation {
    fn name(&self) -> &'static str {
        "Lint/ImplicitStringConcatenation"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["chained_string"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut cur = node.walk();
        let parts: Vec<_> = node.named_children(&mut cur).collect();
        for pair in parts.windows(2) {
            let (lhs, rhs) = (pair[0], pair[1]);
            // RuboCop only flags same-line adjacent literals (multiline `\` is OK).
            if node_line(source, lhs) != node_line(source, rhs) {
                continue;
            }
            if !ends_with_own_delimiter(source, lhs) {
                continue;
            }
            let (line, col) = source.offset_to_line_col(rhs.start_byte());
            diagnostics.push(self.diagnostic(
                source,
                line,
                col,
                format!(
                    "Combine `{}` and `{}` into a single string literal, rather than using implicit string concatenation.",
                    node_text(source, lhs),
                    node_text(source, rhs)
                ),
            ));
        }
    }
}

fn ends_with_own_delimiter(source: &SourceFile, node: Node<'_>) -> bool {
    let bytes = source.as_bytes();
    let Some(&first) = bytes.get(node.start_byte()) else {
        return false;
    };
    let delim = match first {
        b'\'' | b'"' => first,
        _ => return false, // %q etc. — not flagged
    };
    bytes.get(node.end_byte().saturating_sub(1)) == Some(&delim)
}
