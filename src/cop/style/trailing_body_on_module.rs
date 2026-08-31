//! Style/TrailingBodyOnModule — no body on same line as module.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct TrailingBodyOnModule;

impl Cop for TrailingBodyOnModule {
    fn name(&self) -> &'static str {
        "Style/TrailingBodyOnModule"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["module"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        let Some(body) = node.child_by_field_name("body") else {
            return;
        };
        if name.end_position().row != body.start_position().row {
            return;
        }
        let mut cur = body.walk();
        if body.named_children(&mut cur).next().is_none() {
            return;
        }
        let (line, col) = source.offset_to_line_col(body.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Place the body of a module definition on its own line.".to_string(),
        ));
    }
}
