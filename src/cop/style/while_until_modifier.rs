//! Style/WhileUntilModifier — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_while_until_modifier;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct WhileUntilModifier;

impl Cop for WhileUntilModifier {
    fn name(&self) -> &'static str {
        "Style/WhileUntilModifier"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["while_modifier", "until_modifier"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_while_until_modifier(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/WhileUntilModifier offense.".to_string(),
        ));
    }
}
