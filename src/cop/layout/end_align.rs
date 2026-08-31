//! Shared `end` keyword alignment for Layout/*EndAlignment cops.

use tree_sitter::Node;

use crate::cop::shared;
use crate::cop::Cop;
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

fn apply_fix(
    source: &SourceFile,
    end_kw: Node<'_>,
    el: usize,
    base_col: usize,
    cop_name: &'static str,
    corr: &mut Vec<Correction>,
) -> bool {
    let Some(ls) = source.line_start(el) else {
        return false;
    };
    let cur = shared::line_indent(source, end_kw.start_byte());
    corr.push(Correction {
        start: ls,
        end: ls + cur,
        replacement: " ".repeat(base_col),
        cop_name,
        cop_index: 0,
    });
    true
}

fn report_misaligned_end(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    end_kw: Node<'_>,
    base_name: &str,
    base_col: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (el, ec) = source.offset_to_line_col(end_kw.start_byte());
    let (bl, bc) = source.offset_to_line_col(node.start_byte());
    let mut diag = cop.diagnostic(
        source,
        el,
        ec,
        format!("`end` at {el}, {ec} is not aligned with `{base_name}` at {bl}, {bc}."),
    );
    if let Some(corr) = corrections {
        if apply_fix(source, end_kw, el, base_col, cop.name(), corr) {
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

/// Align `end` with the opening keyword named by `base_name`.
pub fn check_end(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    base_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(end_kw) = shared::end_keyword(node) else {
        return;
    };
    let base_col = shared::node_col(source, node);
    if shared::node_col(source, end_kw) == base_col {
        return;
    }
    report_misaligned_end(
        cop, source, node, end_kw, base_name, base_col, diagnostics, corrections,
    );
}
