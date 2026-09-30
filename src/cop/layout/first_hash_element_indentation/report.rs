//! Offense reporting and the send `(` column for Layout/FirstHashElementIndentation.

use tree_sitter::Node;

use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// One line of a pair or `}`, shifted by the same column delta RuboCop applies
/// to every line of the node.
fn line_edit(
    source: &SourceFile,
    line: usize,
    delta: isize,
) -> Option<(usize, usize, String)> {
    let start = source.line_start(line)?;
    let indent = shared::line_indent(source, start);
    let new_indent = (indent as isize + delta).max(0) as usize;
    (new_indent != indent).then(|| (start, start + indent, " ".repeat(new_indent)))
}

fn push_edits(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    delta: isize,
    corrections: &mut Vec<Correction>,
) -> bool {
    let start_line = shared::node_line(source, node);
    let end_line = source
        .offset_to_line_col(node.end_byte().saturating_sub(1))
        .0;
    let mut applied = false;
    for line in start_line..=end_line {
        let Some((start, end, replacement)) = line_edit(source, line, delta) else {
            continue;
        };
        corrections.push(Correction {
            start,
            end,
            replacement,
            cop_name: cop.name(),
            cop_index: 0,
        });
        applied = true;
    }
    applied
}

pub(super) fn report_at(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    expected: usize,
    message: String,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let actual = shared::node_col(source, node);
    if actual == expected {
        return;
    }
    let (line, col) = source.offset_to_line_col(node.start_byte());
    let mut diag = cop.diagnostic(source, line, col, message);
    if let Some(corr) = corrections
        && push_edits(cop, source, node, expected as isize - actual as isize, corr)
    {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn method_call_args(source: &SourceFile, args: Node<'_>) -> bool {
    let Some(call) = args.parent().filter(|p| p.kind() == "call") else {
        return false;
    };
    // `super(` is not a send, so RuboCop keeps the line-start indent.
    call.child_by_field_name("method")
        .is_none_or(|m| shared::node_bytes(source, m) != b"super")
}

fn paren_on_line(source: &SourceFile, node: Node<'_>, line: usize) -> Option<usize> {
    if node.kind() != "argument_list" || !method_call_args(source, node) {
        return None;
    }
    let mut cur = node.walk();
    let paren = node
        .children(&mut cur)
        .find(|c| !c.is_named() && c.kind() == "(")?;
    (shared::node_line(source, paren) == line).then_some(shared::node_col(source, paren))
}

pub(super) fn paren_col(source: &SourceFile, hash: Node<'_>, config: &CopConfig) -> Option<usize> {
    if config.get_bool("FixedArgumentIndentation", false) {
        return None;
    }
    let brace_line = shared::node_line(source, hash);
    let mut cur = hash.parent();
    while let Some(node) = cur {
        if let Some(col) = paren_on_line(source, node, brace_line) {
            return Some(col);
        }
        cur = node.parent();
    }
    None
}
