//! Style/HashSyntax — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_hash_syntax;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct HashSyntax;

impl Cop for HashSyntax {
    fn name(&self) -> &'static str {
        "Style/HashSyntax"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["pair", "hash"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_hash_syntax(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            hash_syntax_message(config),
        ));
    }
}

fn hash_syntax_message(config: &CopConfig) -> String {
    match config.get_str("EnforcedStyle", "ruby19") {
        "hash_rockets" => "Use hash rockets syntax.".to_string(),
        _ => "Use the new Ruby 1.9 hash syntax.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(HashSyntax, "cops/style/hash_syntax");
}
