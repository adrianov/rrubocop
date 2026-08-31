//! Naming/ClassAndModuleCamelCase.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ClassAndModuleCamelCase;

impl Cop for ClassAndModuleCamelCase {
    fn name(&self) -> &'static str {
        "Naming/ClassAndModuleCamelCase"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class", "module"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name_node = if name_node.kind() == "scope_resolution" {
            name_node.child_by_field_name("name").unwrap_or(name_node)
        } else {
            name_node
        };
        let name = node_bytes(source, name_node);
        if is_camel_case(name) {
            return;
        }
        let (line, column) = source.offset_to_line_col(name_node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            "Use CamelCase for classes and modules. (https://rubystyle.guide#camelcase-classes)".into(),
        ));
    }
}

fn is_camel_case(name: &[u8]) -> bool {
    if name.is_empty() || !name[0].is_ascii_uppercase() {
        return false;
    }
    !name.iter().any(|&b| b == b'_')
}
