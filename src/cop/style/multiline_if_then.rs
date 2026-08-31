//! Style/MultilineIfThen — no `then` on multi-line if/unless.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineIfThen;

impl Cop for MultilineIfThen {
    fn name(&self) -> &'static str {
        "Style/MultilineIfThen"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if", "unless"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if node.start_position().row == node.end_position().row {
            return;
        }
        let Some(then_kw) = find_then_keyword(source, node) else {
            return;
        };
        report(self, source, then_kw, diagnostics, &mut corrections);
    }
}

fn report(
    cop: &MultilineIfThen,
    source: &SourceFile,
    then_kw: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(then_kw.start_byte());
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        "Do not use `then` for multi-line `if`/`unless`.".to_string(),
    );
    if let Some(corr) = corrections {
        let src = source.as_bytes();
        let mut remove_start = then_kw.start_byte();
        while remove_start > 0 && src[remove_start - 1] == b' ' {
            remove_start -= 1;
        }
        corr.push(Correction {
            start: remove_start,
            end: then_kw.end_byte(),
            replacement: String::new(),
            cop_name: cop.name(),
            cop_index: 0,
        });
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

/// Locate the `then` keyword token under a multi-line if/unless.
fn find_then_keyword<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        if node_bytes(source, child) == b"then" {
            return Some(child);
        }
        if let Some(kw) = then_in_then_node(source, child) {
            return Some(kw);
        }
    }
    None
}

fn then_in_then_node<'a>(source: &SourceFile, child: Node<'a>) -> Option<Node<'a>> {
    if child.kind() != "then" {
        return None;
    }
    let mut tc = child.walk();
    child
        .children(&mut tc)
        .find(|gc| node_bytes(source, *gc) == b"then")
}
