use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

use std::collections::HashSet;

/// Lint/DuplicateHashKey — duplicate literal keys in hash.
pub struct DuplicateHashKey;

fn key_bytes(source: &SourceFile, key: Node<'_>) -> Option<Vec<u8>> {
    match key.kind() {
        "hash_key_symbol" | "simple_symbol" | "integer" | "float" | "string" | "constant"
        | "true" | "false" | "nil" => Some(node_bytes(source, key).to_vec()),
        "scope_resolution" => Some(node_bytes(source, key).to_vec()),
        _ => None,
    }
}

impl Cop for DuplicateHashKey {
    fn name(&self) -> &'static str {
        "Lint/DuplicateHashKey"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["hash"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut seen = HashSet::new();
        let mut cur = node.walk();
        for child in node.named_children(&mut cur) {
            if child.kind() != "pair" {
                continue;
            }
            let Some(key) = child.child_by_field_name("key") else {
                continue;
            };
            let Some(canon) = key_bytes(source, key) else {
                continue;
            };
            if !seen.insert(canon) {
                let (line, col) = source.offset_to_line_col(key.start_byte());
                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    col,
                    "Duplicated key in hash literal.".to_string(),
                ));
            }
        }
    }
}
