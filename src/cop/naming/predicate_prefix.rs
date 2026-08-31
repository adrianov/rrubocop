//! Naming/PredicatePrefix — forbidden prefixes on ? methods (default: is_).

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct PredicatePrefix;

impl Cop for PredicatePrefix {
    fn name(&self) -> &'static str {
        "Naming/PredicatePrefix"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name = node_bytes(source, name_node);
        if !name.ends_with(b"?") {
            return;
        }
        let Some(prefix) = matching_prefix(name, config.get_str("NamePrefix", "is_")) else {
            return;
        };
        let (line, column) = source.offset_to_line_col(name_node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            format!(
                "Rename `{n}` to remove the `{prefix}` prefix.",
                n = String::from_utf8_lossy(name)
            ),
        ));
    }
}

fn matching_prefix<'a>(name: &[u8], forbidden: &'a str) -> Option<&'a str> {
    forbidden
        .split(',')
        .map(str::trim)
        .find(|p| !p.is_empty() && name.starts_with(p.as_bytes()))
}
