//! Style/GlobalVars — avoid global variables.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct GlobalVars;

impl Cop for GlobalVars {
    fn name(&self) -> &'static str {
        "Style/GlobalVars"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["global_variable"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let name = String::from_utf8_lossy(node_bytes(source, node));
        // Allow English / special globals commonly exempted
        if name.as_bytes().get(1).copied() == Some(b':') {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Do not introduce global variables (`{name}`)."),
        ));
    }
}
