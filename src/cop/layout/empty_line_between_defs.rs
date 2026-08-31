//! Layout/EmptyLineBetweenDefs.

use std::collections::HashMap;
use tree_sitter::{Node, Tree};

use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct EmptyLineBetweenDefs;

fn collect_defs<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
    if matches!(node.kind(), "method" | "singleton_method" | "class" | "module") {
        out.push(node);
    }
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        collect_defs(child, out);
    }
}

fn is_method(n: Node<'_>) -> bool {
    matches!(n.kind(), "method" | "singleton_method")
}

fn expected_label(number: usize) -> String {
    if number == 1 { "1 empty line".into() } else { format!("{number} empty lines") }
}

fn fix_gap(
    cop: &dyn Cop, source: &SourceFile, a_end: usize, b_start: usize, number: usize,
    corr: &mut Vec<Correction>,
) -> bool {
    let Some(insert_at) = source.line_start(a_end + 1) else { return false; };
    let end_at = source.line_start(b_start).unwrap_or(insert_at);
    corr.push(Correction {
        start: insert_at, end: end_at, replacement: "\n".repeat(number),
        cop_name: cop.name(), cop_index: 0,
    });
    true
}

fn gap_blanks(source: &SourceFile, a: Node<'_>, b: Node<'_>) -> (usize, usize, usize) {
    let (a_end, _) = source.offset_to_line_col(a.end_byte().saturating_sub(1));
    let b_start = shared::node_line(source, b);
    let blanks = b_start.saturating_sub(a_end + 1).min(100);
    (a_end, b_start, blanks)
}

fn report_gap(
    cop: &dyn Cop, source: &SourceFile, b: Node<'_>, a_end: usize, b_start: usize,
    number: usize, blanks: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(b.start_byte());
    let expected = expected_label(number);
    let mut diag = cop.diagnostic(
        source, line, col,
        format!("Expected {expected} between method definitions; found {blanks}."),
    );
    if let Some(corr) = corrections {
        if fix_gap(cop, source, a_end, b_start, number, corr) {
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

fn check_pair(
    cop: &dyn Cop, source: &SourceFile, a: Node<'_>, b: Node<'_>, number: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !is_method(a) || !is_method(b) { return; }
    let (a_end, b_start, blanks) = gap_blanks(source, a, b);
    if blanks == number { return; }
    report_gap(cop, source, b, a_end, b_start, number, blanks, diagnostics, corrections);
}

fn check_siblings(
    cop: &dyn Cop, source: &SourceFile, siblings: &[Node<'_>], number: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut sorted = siblings.to_vec();
    sorted.sort_by_key(|n| n.start_byte());
    for w in sorted.windows(2) {
        check_pair(cop, source, w[0], w[1], number, diagnostics, corrections);
    }
}

impl Cop for EmptyLineBetweenDefs {
    fn name(&self) -> &'static str { "Layout/EmptyLineBetweenDefs" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, tree: &Tree, _code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let number = config.get_usize("NumberOfEmptyLines", 1);
        let mut defs = Vec::new();
        collect_defs(tree.root_node(), &mut defs);
        let mut by_parent: HashMap<usize, Vec<Node<'_>>> = HashMap::new();
        for d in defs {
            let key = d.parent().map(|p| p.id()).unwrap_or(0);
            by_parent.entry(key).or_default().push(d);
        }
        for siblings in by_parent.values() {
            check_siblings(self, source, siblings, number, diagnostics, &mut corrections);
        }
    }
}
