//! Shared per-item line-break checks for Layout/Multiline*LineBreaks cops.

use tree_sitter::Node;

use crate::cop::shared;
use crate::cop::Cop;
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

fn report_same_line(
    cop: &dyn Cop,
    source: &SourceFile,
    item: Node<'_>,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(item.start_byte());
    let mut diag = cop.diagnostic(source, line, col, message.into());
    if let Some(corr) = corrections {
        corr.push(Correction {
            start: item.start_byte(),
            end: item.start_byte(),
            replacement: "\n".into(),
            cop_name: cop.name(),
            cop_index: 0,
        });
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

/// Require each named child on its own line when the parent spans lines.
pub fn check_breaks(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut cur = node.walk();
    let elems: Vec<_> = node.named_children(&mut cur).collect();
    if elems.len() < 2 {
        return;
    }
    let start_line = shared::node_line(source, node);
    let (end_line, _) = source.offset_to_line_col(node.end_byte().saturating_sub(1));
    if start_line == end_line {
        return;
    }
    for w in elems.windows(2) {
        if shared::node_line(source, w[0]) == shared::node_line(source, w[1]) {
            report_same_line(cop, source, w[1], message, diagnostics, corrections);
        }
    }
}
