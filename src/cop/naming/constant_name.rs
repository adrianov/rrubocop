//! Naming/ConstantName — constants must be SCREAMING_SNAKE_CASE.

use tree_sitter::Node;

use crate::cop::shared::{is_screaming_snake_case, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ConstantName;

impl Cop for ConstantName {
    fn name(&self) -> &'static str {
        "Naming/ConstantName"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["assignment", "operator_assignment"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(left) = node.child_by_field_name("left") else {
            return;
        };
        let const_node = match left.kind() {
            "constant" => left,
            "scope_resolution" => left.child_by_field_name("name").unwrap_or(left),
            _ => return,
        };
        if const_node.kind() != "constant" {
            return;
        }
        let name = node_bytes(source, const_node);
        if is_screaming_snake_case(name) {
            return;
        }
        let (line, column) = source.offset_to_line_col(const_node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            format!(
                "Use SCREAMING_SNAKE_CASE for constants. (https://rubystyle.guide#screaming-snake-case-constants)"
            ),
        ));
    }
}
