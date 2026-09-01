//! Report indent offenses for multiline binary operations.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

use super::indent::{indent_mismatch, not_for_this_cop};

pub(super) fn check_binary(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if not_for_this_cop(node) {
        return;
    }
    let Some((actual, expected, align_only)) = indent_mismatch(source, node, config) else {
        return;
    };
    let right = node.child_by_field_name("right").unwrap();
    let left = node.child_by_field_name("left").unwrap();
    let msg = offense_message(source, left, actual, expected, align_only);
    report::fix_indent(
        cop,
        source,
        right.start_byte(),
        msg,
        diagnostics,
        corrections,
        actual,
        expected,
    );
}

fn offense_message(
    source: &SourceFile,
    left: Node<'_>,
    actual: usize,
    expected: usize,
    align_only: bool,
) -> String {
    if align_only {
        return "Align the operands of a multi-line operation.".into();
    }
    let left_indent = shared::line_indent(source, left.start_byte());
    let used = actual.saturating_sub(left_indent);
    let want = expected.saturating_sub(left_indent);
    format!("Use {want} (not {used}) spaces for indenting a multi-line operation.")
}
