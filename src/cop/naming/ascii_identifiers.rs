//! Naming/AsciiIdentifiers — non-ASCII in identifiers.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct AsciiIdentifiers;

impl Cop for AsciiIdentifiers {
    fn name(&self) -> &'static str {
        "Naming/AsciiIdentifiers"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &[
            "identifier",
            "constant",
            "instance_variable",
            "class_variable",
            "global_variable",
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let bytes = &source.as_bytes()[node.start_byte()..node.end_byte()];
        if bytes.iter().all(|&b| b.is_ascii()) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Use only ascii characters in identifiers.".to_string(),
        ));
    }
}
