use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/SuppressedException — empty rescue body.
pub struct SuppressedException;

enum RescueBody {
    Empty,
    OnlyNil,
    Other,
}

fn stmts_of(body: Node<'_>) -> Vec<Node<'_>> {
    let mut cur = body.walk();
    body.named_children(&mut cur).collect()
}

fn classify_stmts(stmts: &[Node<'_>]) -> RescueBody {
    if stmts.is_empty() {
        return RescueBody::Empty;
    }
    // `;` alone / empty statements count as empty rescue bodies.
    let meaningful: Vec<_> = stmts
        .iter()
        .copied()
        .filter(|n| !matches!(n.kind(), "empty_statement" | ";"))
        .collect();
    if meaningful.is_empty() {
        RescueBody::Empty
    } else if meaningful.len() == 1 && meaningful[0].kind() == "nil" {
        RescueBody::OnlyNil
    } else {
        RescueBody::Other
    }
}

fn classify_body(node: Node<'_>) -> RescueBody {
    if let Some(body) = node.child_by_field_name("body") {
        return classify_stmts(&stmts_of(body));
    }
    let mut cur = node.walk();
    let named: Vec<_> = node
        .named_children(&mut cur)
        .filter(|n| !matches!(n.kind(), "exceptions" | "exception_variable"))
        .collect();
    if named.len() == 1 && named[0].kind() == "then" {
        return classify_stmts(&stmts_of(named[0]));
    }
    classify_stmts(&named)
}

impl Cop for SuppressedException {
    fn name(&self) -> &'static str {
        "Lint/SuppressedException"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
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
        let allow_nil = config.get_bool("AllowNil", true);
        match classify_body(node) {
            RescueBody::Other => return,
            RescueBody::OnlyNil if allow_nil => return,
            RescueBody::Empty | RescueBody::OnlyNil => {}
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not suppress exceptions.".to_string(),
        ));
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(SuppressedException, "cops/lint/suppressed_exception");
}
