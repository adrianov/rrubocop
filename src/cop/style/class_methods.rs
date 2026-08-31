//! Style/ClassMethods — prefer def self.x over class << self.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ClassMethods;

impl Cop for ClassMethods {
    fn name(&self) -> &'static str {
        "Style/ClassMethods"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["singleton_class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Use `def self.method` to define class methods.".to_string(),
        ));
    }
}
