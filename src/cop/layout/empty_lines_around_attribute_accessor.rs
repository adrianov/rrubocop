//! Layout/EmptyLinesAroundAttributeAccessor.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLinesAroundAttributeAccessor;

fn is_attr(name: &[u8]) -> bool {
    matches!(name, b"attr_reader" | b"attr_writer" | b"attr_accessor" | b"attr")
}

impl Cop for EmptyLinesAroundAttributeAccessor {
    fn name(&self) -> &'static str { "Layout/EmptyLinesAroundAttributeAccessor" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["call", "command"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        let Some(name) = shared::call_method_name(source, node) else { return; };
        if !is_attr(name) { return; }
        let (end_line, _) = source.offset_to_line_col(node.end_byte().saturating_sub(1));
        let next = end_line + 1;
        if shared::line_blank(source, next) || source.line_start(next).is_none() { return; }
        report::insert_newline(
            self, source, next,
            "Add an empty line after attribute accessor.".into(),
            diagnostics, &mut corrections,
        );
    }
}
