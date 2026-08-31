//! RSpec/SubjectDeclaration — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct SubjectDeclaration;

const MSG: &str = "Use subject explicitly rather than using let";

impl Cop for SubjectDeclaration {
    fn name(&self) -> &'static str {
        "RSpec/SubjectDeclaration"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "hash", "string", "symbol", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        const METHODS: &[&[u8]] = &[b"subject"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
