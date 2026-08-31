//! Rails/PluralizationGrammar — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct PluralizationGrammar;

const MSG: &str = "Prefer `.......`.";

impl Cop for PluralizationGrammar {
    fn name(&self) -> &'static str {
        "Rails/PluralizationGrammar"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "integer", "command"]
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
        const METHODS: &[&[u8]] = &[b"byte", b"bytes", b"kilobyte", b"kilobytes", b"megabyte", b"megabytes", b"gigabyte", b"gigabytes", b"terabyte", b"terabytes"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
