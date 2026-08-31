//! Style/EmptyCaseCondition — case without condition.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyCaseCondition;

impl Cop for EmptyCaseCondition {
    fn name(&self) -> &'static str {
        "Style/EmptyCaseCondition"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["case"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.child_by_field_name("value").is_some() {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not use empty `case` condition, instead use an `if` expression.".to_string(),
        ));
    }
}
