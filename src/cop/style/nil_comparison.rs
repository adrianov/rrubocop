//! Style/NilComparison — prefer nil? over == nil / != nil.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct NilComparison;

impl Cop for NilComparison {
    fn name(&self) -> &'static str {
        "Style/NilComparison"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["binary"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some((op, left, right, kids)) = detect(source, node) else {
            return;
        };
        report(self, source, node, op, left, right, &kids, diagnostics, &mut corrections);
    }
}

fn detect<'a>(
    source: &'a SourceFile,
    node: Node<'a>,
) -> Option<(&'a [u8], &'a [u8], &'a [u8], Vec<Node<'a>>)> {
    let mut cur = node.walk();
    let kids: Vec<_> = node.children(&mut cur).collect();
    if kids.len() < 3 {
        return None;
    }
    let op = node_bytes(source, kids[1]);
    if op != b"==" && op != b"!=" {
        return None;
    }
    let left = node_bytes(source, kids[0]);
    let right = node_bytes(source, kids[2]);
    if left != b"nil" && right != b"nil" {
        return None;
    }
    Some((op, left, right, kids))
}

fn report(
    cop: &NilComparison,
    source: &SourceFile,
    node: Node<'_>,
    op: &[u8],
    left: &[u8],
    right: &[u8],
    kids: &[Node<'_>],
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(node.start_byte());
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        "Prefer the use of the `nil?` predicate.".to_string(),
    );
    if let Some(corr) = corrections {
        let (start, end, replacement) = correction(node, op, left, right, kids);
        corr.push(Correction {
            start,
            end,
            replacement,
            cop_name: cop.name(),
            cop_index: 0,
        });
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn correction(
    node: Node<'_>,
    op: &[u8],
    left: &[u8],
    right: &[u8],
    kids: &[Node<'_>],
) -> (usize, usize, String) {
    if right == b"nil" && op == b"==" {
        return (kids[0].end_byte(), node.end_byte(), ".nil?".to_string());
    }
    let other = if right == b"nil" {
        String::from_utf8_lossy(left).into_owned()
    } else {
        String::from_utf8_lossy(right).into_owned()
    };
    let replacement = if op == b"!=" {
        format!("!{other}.nil?")
    } else {
        format!("{other}.nil?")
    };
    (node.start_byte(), node.end_byte(), replacement)
}
