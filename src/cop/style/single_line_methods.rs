//! Style/SingleLineMethods — avoid single-line methods.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SingleLineMethods;

fn method_body(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("body").or_else(|| {
        let mut cur = node.walk();
        node.named_children(&mut cur)
            .find(|n| n.kind() == "body_statement")
    })
}

fn is_endless_method(source: &SourceFile, node: Node<'_>) -> bool {
    // `def name(...) = expr` — no `end`, `=` after parameters/name.
    if shared_has_end(node) {
        return false;
    }
    let bytes = source.as_bytes();
    let start = node.start_byte();
    let end = node.end_byte().min(bytes.len());
    bytes[start..end].contains(&b'=')
}

fn shared_has_end(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.children(&mut cur).any(|c| !c.is_named() && c.kind() == "end")
}

fn body_is_empty(body: Node<'_>) -> bool {
    let mut cur = body.walk();
    !body
        .named_children(&mut cur)
        .any(|n| n.kind() != "comment")
}

impl Cop for SingleLineMethods {
    fn name(&self) -> &'static str {
        "Style/SingleLineMethods"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.start_position().row != node.end_position().row {
            return;
        }
        // RuboCop skips endless methods (`def foo = bar`).
        if is_endless_method(source, node) {
            return;
        }
        let allow_empty = config.get_bool("AllowIfMethodIsEmpty", true);
        if allow_empty {
            match method_body(node) {
                None => return,
                Some(body) if body_is_empty(body) => return,
                Some(_) => {}
            }
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Avoid single-line method definitions.".to_string(),
        ));
    }
}
