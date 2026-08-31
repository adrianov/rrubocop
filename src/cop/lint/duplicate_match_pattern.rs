use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

use std::collections::HashSet;

/// Lint/DuplicateMatchPattern — duplicate `in` patterns in `case`.
pub struct DuplicateMatchPattern;

impl Cop for DuplicateMatchPattern {
    fn name(&self) -> &'static str {
        "Lint/DuplicateMatchPattern"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["case_match"]
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
            if child.kind() != "in_clause" {
                continue;
            }
            let Some(pat) = child.child_by_field_name("pattern") else {
                continue;
            };
            let key = node_bytes(source, pat).to_vec();
            if !seen.insert(key) {
                let (line, col) = source.offset_to_line_col(pat.start_byte());
                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    col,
                    "Duplicate `in` pattern detected.".to_string(),
                ));
            }
        }
    }
}
