//! Naming/AccessorMethodName — prefer attr_* over get_/set_.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct AccessorMethodName;

impl Cop for AccessorMethodName {
    fn name(&self) -> &'static str {
        "Naming/AccessorMethodName"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
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
        let name = node_bytes(source, name_node);
        let msg = if name.starts_with(b"get_") {
            "Do not prefix reader method names with `get_`. (https://rubystyle.guide#accessor_methods)"
        } else if name.starts_with(b"set_") {
            "Do not prefix writer method names with `set_`. (https://rubystyle.guide#accessor_methods)"
        } else {
            return;
        };
        let (line, column) = source.offset_to_line_col(name_node.start_byte());
        diagnostics.push(self.diagnostic(source, line, column, msg.into()));
    }
}
