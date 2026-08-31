//! Style/IfUnlessModifierOfIfUnless.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct IfUnlessModifierOfIfUnless;

impl Cop for IfUnlessModifierOfIfUnless {
    fn name(&self) -> &'static str {
        "Style/IfUnlessModifierOfIfUnless"
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
        if !matches!(body.kind(), "if" | "unless" | "if_modifier" | "unless_modifier") {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Avoid modifier `if`/`unless` used with another `if`/`unless`.".to_string(),
        ));
    }
}
