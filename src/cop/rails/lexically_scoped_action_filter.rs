//! Rails/LexicallyScopedActionFilter — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct LexicallyScopedActionFilter;

const MSG: &str = "Rails/LexicallyScopedActionFilter offense.";

impl Cop for LexicallyScopedActionFilter {
    fn name(&self) -> &'static str {
        "Rails/LexicallyScopedActionFilter"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/app/controllers/**/*.rb", "**/app/mailers/**/*.rb"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["array", "pair", "class", "method", "hash", "module", "body_statement", "string", "symbol"]
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
        const METHODS: &[&[u8]] = &[b"after_action", b"append_after_action", b"append_around_action", b"append_before_action", b"around_action", b"before_action"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
