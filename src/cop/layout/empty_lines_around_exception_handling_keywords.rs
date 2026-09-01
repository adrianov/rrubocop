//! Layout/EmptyLinesAroundExceptionHandlingKeywords.

use tree_sitter::{Node, Tree};

use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct EmptyLinesAroundExceptionHandlingKeywords;

fn remove_blank_line(
    cop: &dyn Cop, source: &SourceFile, line: usize, msg: String,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut diag = cop.diagnostic(source, line, 0, msg);
    if let Some(corr) = corrections {
        if let Some(s) = source.line_start(line) {
            let e = source.line_start(line + 1).unwrap_or(s);
            corr.push(Correction {
                start: s, end: e, replacement: String::new(),
                cop_name: cop.name(), cop_index: 0,
            });
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

fn check_before(
    cop: &dyn Cop, source: &SourceFile, line: usize, kw: &str,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if line <= 1 || !shared::line_blank(source, line - 1) { return; }
    remove_blank_line(
        cop, source, line - 1,
        format!("Extra empty line detected before the `{kw}`."),
        diagnostics, corrections,
    );
}

fn check_after(
    cop: &dyn Cop, source: &SourceFile, n: Node<'_>, line: usize, kw: &str,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !shared::line_blank(source, line + 1) { return; }
    let (end_l, _) = source.offset_to_line_col(n.end_byte().saturating_sub(1));
    if end_l <= line + 1 { return; }
    remove_blank_line(
        cop, source, line + 1,
        format!("Extra empty line detected after the `{kw}`."),
        diagnostics, corrections,
    );
}

fn check_kw(
    cop: &dyn Cop, source: &SourceFile, n: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !matches!(n.kind(), "rescue" | "ensure" | "else") { return; }
    // RuboCop only flags `else` inside begin/rescue — not if/case else.
    if n.kind() == "else" {
        let Some(parent) = n.parent() else { return; };
        if !matches!(parent.kind(), "begin" | "body_statement") {
            return;
        }
        let mut cur = parent.walk();
        if !parent.named_children(&mut cur).any(|c| c.kind() == "rescue") {
            return;
        }
    }
    let line = shared::node_line(source, n);
    let kw = n.kind();
    check_before(cop, source, line, kw, diagnostics, corrections);
    check_after(cop, source, n, line, kw, diagnostics, corrections);
}

impl Cop for EmptyLinesAroundExceptionHandlingKeywords {
    fn name(&self) -> &'static str { "Layout/EmptyLinesAroundExceptionHandlingKeywords" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, tree: &Tree, code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = (code_map, config);
        shared::for_each_descendant(tree.root_node(), |n| {
            check_kw(self, source, n, diagnostics, &mut corrections);
        });
    }
}
