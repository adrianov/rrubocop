//! Rails/NotNullColumn — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Severity, Diagnostic};
use crate::parse::source::SourceFile;

pub struct NotNullColumn;


impl Cop for NotNullColumn {
    fn name(&self) -> &'static str {
        "Rails/NotNullColumn"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["db/migrate/**/*.rb"]
    }


    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Breadth-first stub: not implemented — avoid method-name false positives.
        let _ = (source, node, diagnostics);
    }
}
