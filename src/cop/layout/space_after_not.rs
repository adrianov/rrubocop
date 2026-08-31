//! Layout/SpaceAfterNot — ported from RuboCop/nitrocop (tree-sitter).

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceAfterNot;

impl Cop for SpaceAfterNot {
    fn name(&self) -> &'static str { "Layout/SpaceAfterNot" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["unary"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let bytes = source.as_bytes();
        let Some(op) = node.child_by_field_name("operator") else { return; };
        if &bytes[op.start_byte()..op.end_byte()] != b"!" { return; }
        let Some(operand) = node.child_by_field_name("operand") else { return; };
        let bang_end = op.end_byte();
        let recv = operand.start_byte();
        if recv <= bang_end || !bytes[bang_end..recv].iter().any(|b| b.is_ascii_whitespace()) {
            return;
        }
        report::report_fix(
            self, source, op.start_byte(),
            "Do not leave space between `!` and its argument.".into(),
            diagnostics, &mut corrections, bang_end, recv, String::new(),
        );
    }
}
