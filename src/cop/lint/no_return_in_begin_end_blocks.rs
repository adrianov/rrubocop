use tree_sitter::Node;

use crate::cop::shared::for_each_descendant;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/NoReturnInBeginEndBlocks — return inside begin assigned.
pub struct NoReturnInBeginEndBlocks;

fn inside_nested_scope(ret: Node<'_>, begin: Node<'_>) -> bool {
    let mut p = ret.parent();
    while let Some(parent) = p {
        if parent.id() == begin.id() {
            return false;
        }
        if matches!(
            parent.kind(),
            "method" | "singleton_method" | "block" | "do_block"
        ) {
            return true;
        }
        p = parent.parent();
    }
    false
}

fn report_returns(
    cop: &NoReturnInBeginEndBlocks,
    source: &SourceFile,
    begin: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for_each_descendant(begin, |n| {
        if n.kind() != "return" || inside_nested_scope(n, begin) {
            return;
        }
        let (line, col) = source.offset_to_line_col(n.start_byte());
        diagnostics.push(cop.diagnostic(
            source,
            line,
            col,
            "Do not `return` in `begin..end` blocks in assignment contexts.".to_string(),
        ));
    });
}

impl Cop for NoReturnInBeginEndBlocks {
    fn name(&self) -> &'static str {
        "Lint/NoReturnInBeginEndBlocks"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
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
        let Some(right) = node.child_by_field_name("right") else {
            return;
        };
        if right.kind() != "begin" {
            return;
        }
        report_returns(self, source, right, diagnostics);
    }
}
