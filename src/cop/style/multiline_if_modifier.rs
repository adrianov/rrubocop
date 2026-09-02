//! Style/MultilineIfModifier — no multiline bodies on if/unless modifiers.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineIfModifier;

fn modifier_keyword(node: Node<'_>) -> &'static str {
    if node.kind() == "unless_modifier" {
        "unless"
    } else {
        "if"
    }
}

impl Cop for MultilineIfModifier {
    fn name(&self) -> &'static str {
        "Style/MultilineIfModifier"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if_modifier", "unless_modifier"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(body) = node.child_by_field_name("body") else {
            return;
        };
        if body.start_position().row == body.end_position().row {
            return;
        }
        let kw = modifier_keyword(node);
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Favor a normal {kw}-statement over a modifier clause in a multiline statement."),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(MultilineIfModifier, "cops/style/multiline_if_modifier");
}
