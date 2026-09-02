//! Style/RescueStandardError — explicit vs implicit StandardError rescue.

use tree_sitter::Node;

use crate::cop::shared::is_const_named;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RescueStandardError;

fn bare_rescue(node: Node<'_>) -> bool {
    node.child_by_field_name("exceptions").is_none()
}

fn only_standard_error(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(ex) = node.child_by_field_name("exceptions") else {
        return false;
    };
    let mut cur = ex.walk();
    let names: Vec<_> = ex.named_children(&mut cur).collect();
    names.len() == 1 && is_const_named(source, names[0], b"StandardError")
}

impl Cop for RescueStandardError {
    fn name(&self) -> &'static str {
        "Style/RescueStandardError"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["rescue"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.parent().is_some_and(|p| p.kind() == "rescue_modifier") {
            return;
        }
        let style = config.get_str("EnforcedStyle", "explicit");
        let msg = match style {
            "implicit" if only_standard_error(source, node) => {
                Some("Omit the error class when rescuing `StandardError` by itself.")
            }
            "explicit" if bare_rescue(node) => {
                Some("Avoid rescuing without specifying an error class.")
            }
            _ => None,
        };
        let Some(msg) = msg else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, msg.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RescueStandardError, "cops/style/rescue_standard_error");
}
