//! Style/NestedModifier.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct NestedModifier;

const MODS: &[&str] = &[
    "if_modifier",
    "unless_modifier",
    "while_modifier",
    "until_modifier",
];

impl Cop for NestedModifier {
    fn name(&self) -> &'static str {
        "Style/NestedModifier"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        MODS
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !has_nested_mod(node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Avoid using nested modifiers.".to_string(),
        ));
    }
}

fn has_nested_mod(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    if node.named_children(&mut cur).any(|c| MODS.contains(&c.kind())) {
        return true;
    }
    node.child_by_field_name("body")
        .is_some_and(|body| MODS.contains(&body.kind()))
}
