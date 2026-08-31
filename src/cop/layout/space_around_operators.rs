//! Layout/SpaceAroundOperators.

use tree_sitter::{Node, Tree};

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct SpaceAroundOperators;

fn space_ok(slice: &[u8]) -> bool {
    slice == b" " || slice.iter().any(|&b| b == b'\n')
}

fn sides<'a>(n: Node<'a>) -> Option<(Node<'a>, Node<'a>, Node<'a>)> {
    Some((
        n.child_by_field_name("left")?,
        n.child_by_field_name("right")?,
        n.child_by_field_name("operator")?,
    ))
}

fn report_op(
    cop: &dyn Cop,
    source: &SourceFile,
    left: Node<'_>,
    right: Node<'_>,
    op: Node<'_>,
    op_bytes: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let op_s = String::from_utf8_lossy(op_bytes);
    report::report_fix(
        cop,
        source,
        op.start_byte(),
        format!("Operator `{op_s}` should be surrounded by spaces."),
        diagnostics,
        corrections,
        left.end_byte(),
        right.start_byte(),
        format!(" {op_s} "),
    );
}

fn check_binary(
    cop: &dyn Cop,
    source: &SourceFile,
    bytes: &[u8],
    n: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !matches!(n.kind(), "binary" | "operator_assignment") {
        return;
    }
    let Some((left, right, op)) = sides(n) else {
        return;
    };
    let op_bytes = &bytes[op.start_byte()..op.end_byte()];
    if op_bytes == b"**" {
        return;
    }
    let before = &bytes[left.end_byte()..op.start_byte()];
    let after = &bytes[op.end_byte()..right.start_byte()];
    if space_ok(before) && space_ok(after) {
        return;
    }
    report_op(cop, source, left, right, op, op_bytes, diagnostics, corrections);
}

impl Cop for SpaceAroundOperators {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundOperators"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = (code_map, config);
        let bytes = source.as_bytes();
        shared::for_each_descendant(tree.root_node(), |n| {
            check_binary(self, source, bytes, n, diagnostics, &mut corrections);
        });
    }
}
