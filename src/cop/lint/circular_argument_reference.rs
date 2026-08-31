use tree_sitter::Node;

use crate::cop::shared::{node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/CircularArgumentReference — default arg refers to itself.
pub struct CircularArgumentReference;

impl Cop for CircularArgumentReference {
    fn name(&self) -> &'static str {
        "Lint/CircularArgumentReference"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["optional_parameter", "keyword_parameter"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        let Some(value) = node.child_by_field_name("value") else {
            return;
        };
        if value.kind() != "identifier" {
            return;
        }
        if node_bytes(source, name) != node_bytes(source, value) {
            return;
        }
        let arg = node_text(source, name);
        let (line, col) = source.offset_to_line_col(value.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Circular argument reference - `{arg}`."),
        ));
    }
}
