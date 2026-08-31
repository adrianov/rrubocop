//! Style/CommandLiteral — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_command_literal;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CommandLiteral;

impl Cop for CommandLiteral {
    fn name(&self) -> &'static str {
        "Style/CommandLiteral"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["string", "subshell"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_command_literal(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/CommandLiteral offense.".to_string(),
        ));
    }
}
