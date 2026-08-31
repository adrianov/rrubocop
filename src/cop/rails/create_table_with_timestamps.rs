//! Rails/CreateTableWithTimestamps — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct CreateTableWithTimestamps;


impl Cop for CreateTableWithTimestamps {
    fn name(&self) -> &'static str {
        "Rails/CreateTableWithTimestamps"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["db/**/*.rb"]
    }

    fn default_exclude(&self) -> &'static [&'static str] {
        &["**/db/**/*_create_active_storage_tables.active_storage.rb", "**/db/**/*_create_active_storage_variant_records.active_storage.rb"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["pair", "block", "call", "false", "hash", "string", "symbol", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Breadth-first stub: not implemented — avoid method-name false positives.
        let _ = (source, node, diagnostics);
    }
}
