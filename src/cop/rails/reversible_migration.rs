//! Rails/ReversibleMigration — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct ReversibleMigration;

const MSG: &str = "Rails/ReversibleMigration offense.";

impl Cop for ReversibleMigration {
    fn name(&self) -> &'static str {
        "Rails/ReversibleMigration"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["db/**/*.rb"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["pair", "block", "call", "class", "method", "hash", "body_statement", "symbol", "command"]
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
        const METHODS: &[&[u8]] = &[b"execute", b"change_column", b"drop_table", b"remove_column", b"remove_columns", b"remove_index"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
