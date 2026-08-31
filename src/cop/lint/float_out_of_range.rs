use tree_sitter::Node;

use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/FloatOutOfRange — float literal that overflows to Infinity.
pub struct FloatOutOfRange;

impl Cop for FloatOutOfRange {
    fn name(&self) -> &'static str {
        "Lint/FloatOutOfRange"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["float"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let text = node_text(source, node).replace('_', "");
        if let Ok(v) = text.parse::<f64>() {
            if v.is_infinite() {
                let (line, col) = source.offset_to_line_col(node.start_byte());
                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    col,
                    "Float out of range.".to_string(),
                ));
            }
        }
    }
}
