//! Layout/SpaceAfterMethodName — ported from RuboCop/nitrocop (tree-sitter).

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceAfterMethodName;

impl Cop for SpaceAfterMethodName {
    fn name(&self) -> &'static str { "Layout/SpaceAfterMethodName" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(name) = node.child_by_field_name("name") else { return; };
        let Some(params) = node.child_by_field_name("parameters") else { return; };
        let bytes = source.as_bytes();
        if bytes.get(params.start_byte()) != Some(&b'(') { return; }
        let name_end = name.end_byte();
        let lparen = params.start_byte();
        if lparen <= name_end || !bytes[name_end..lparen].iter().any(|&b| b == b' ' || b == b'\t') {
            return;
        }
        report::report_fix(
            self, source, name_end,
            "Do not put a space between a method name and the opening parenthesis.".into(),
            diagnostics, &mut corrections, name_end, lparen, String::new(),
        );
    }
}
