//! Style/RedundantReturn — avoid unnecessary return.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RedundantReturn;

impl Cop for RedundantReturn {
    fn name(&self) -> &'static str {
        "Style/RedundantReturn"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["return"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if !is_trailing_return(node) {
            return;
        }
        report(self, source, node, diagnostics, &mut corrections);
    }
}

fn report(
    cop: &RedundantReturn,
    source: &SourceFile,
    node: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(node.start_byte());
    let mut diag = cop.diagnostic(source, line, col, "Redundant `return` detected.".to_string());
    if let Some(corr) = corrections.as_mut() {
        push_remove_return(cop, node, corr);
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn push_remove_return(cop: &RedundantReturn, node: Node<'_>, corr: &mut Vec<Correction>) {
    let mut cur = node.walk();
    let named: Vec<_> = node.named_children(&mut cur).collect();
    let end = if named.is_empty() {
        node.end_byte()
    } else {
        named[0].start_byte()
    };
    corr.push(Correction {
        start: node.start_byte(),
        end,
        replacement: String::new(),
        cop_name: cop.name(),
        cop_index: 0,
    });
}

fn is_trailing_return(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    let Some(gp) = parent.parent() else {
        return false;
    };
    let container = if parent.kind() == "body_statement" {
        gp
    } else {
        parent
    };
    if !matches!(
        container.kind(),
        "method" | "singleton_method"
    ) {
        // `return` inside a block exits the enclosing method — never redundant.
        return false;
    }
    let Some(body) = return_body(parent, container) else {
        return false;
    };
    let mut cur = body.walk();
    let named: Vec<_> = body.named_children(&mut cur).collect();
    named.last().map(|n| n.id()) == Some(node.id())
}

fn return_body<'a>(parent: Node<'a>, container: Node<'a>) -> Option<Node<'a>> {
    if parent.kind() == "body_statement" {
        Some(parent)
    } else {
        container.child_by_field_name("body")
    }
}
