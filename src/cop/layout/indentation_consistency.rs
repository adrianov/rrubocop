//! Layout/IndentationConsistency.

use tree_sitter::{Node, Tree};

use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct IndentationConsistency;

fn is_container(kind: &str) -> bool {
    matches!(kind, "class" | "module" | "method" | "singleton_method" | "begin" | "do_block")
}

fn skip_kid(kind: &str) -> bool {
    matches!(kind, "identifier" | "constant" | "superclass" | "method_parameters" | "parameters" | "block_parameters")
}

fn nested_ok(kind: &str) -> bool {
    matches!(
        kind,
        "method" | "class" | "module" | "singleton_method" | "do_block" | "block" | "begin"
            | "if" | "unless" | "while" | "until" | "case" | "for"
    )
}

fn body_kids<'a>(n: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = n.walk();
    n.named_children(&mut cur).filter(|k| !skip_kid(k.kind())).collect()
}

fn needs_fix(ind: usize, base: usize, kind: &str) -> bool {
    ind < base || (ind > base && !nested_ok(kind))
}

fn fix_indent(
    source: &SourceFile, offset: usize, base: usize, cop_name: &'static str, corr: &mut Vec<Correction>,
) -> bool {
    let (line, _) = source.offset_to_line_col(offset);
    let Some(ls) = source.line_start(line) else { return false; };
    let cur = shared::line_indent(source, offset);
    if cur == base { return false; }
    corr.push(Correction {
        start: ls, end: ls + cur, replacement: " ".repeat(base),
        cop_name, cop_index: 0,
    });
    true
}

fn check_kid(
    cop: &dyn Cop, source: &SourceFile, first: Node<'_>, k: Node<'_>, base: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let ind = shared::line_indent(source, k.start_byte());
    if ind == base || shared::node_line(source, k) == shared::node_line(source, first) { return; }
    if !needs_fix(ind, base, k.kind()) { return; }
    let (l, c) = source.offset_to_line_col(k.start_byte());
    let mut diag = cop.diagnostic(source, l, c, "Inconsistent indentation detected.".into());
    if let Some(corr) = corrections {
        if fix_indent(source, k.start_byte(), base, cop.name(), corr) {
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

fn check_container(
    cop: &dyn Cop, source: &SourceFile, n: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !is_container(n.kind()) { return; }
    let kids = body_kids(n);
    if kids.len() < 2 { return; }
    let base = shared::line_indent(source, kids[0].start_byte());
    for k in kids.iter().skip(1) {
        check_kid(cop, source, kids[0], *k, base, diagnostics, corrections);
    }
}

impl Cop for IndentationConsistency {
    fn name(&self) -> &'static str { "Layout/IndentationConsistency" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, tree: &Tree, _code_map: &CodeMap,
        _config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        shared::for_each_descendant(tree.root_node(), |n| {
            check_container(self, source, n, diagnostics, &mut corrections);
        });
    }
}
