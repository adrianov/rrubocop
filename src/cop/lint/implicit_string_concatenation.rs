use tree_sitter::Node;

use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/ImplicitStringConcatenation — adjacent string literals.
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
        if parts.len() < 2 {
            return;
        }
        let lhs = node_text(source, parts[0]);
        let rhs = node_text(source, parts[1]);
        let (line, col) = source.offset_to_line_col(parts[1].start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!(
                "Combine `{lhs}` and `{rhs}` into a single string literal, rather than using implicit string concatenation."
            ),
        ));
    }
}
