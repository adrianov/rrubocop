//! Rails/SkipsModelValidations — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct SkipsModelValidations;

const MSG: &str = "Rails/SkipsModelValidations offense.";

impl Cop for SkipsModelValidations {
    fn name(&self) -> &'static str {
        "Rails/SkipsModelValidations"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["pair", "call", "scope_resolution", "constant", "false", "hash", "symbol", "true", "command"]
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
        const METHODS: &[&[u8]] = &[b"update_attribute", b"update_column", b"update_columns", b"update_all", b"upsert", b"upsert_all", b"increment!", b"decrement!", b"toggle!", b"increment_counter", b"decrement_counter"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
