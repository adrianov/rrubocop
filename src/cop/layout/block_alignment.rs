//! Layout/BlockAlignment.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BlockAlignment;

impl Cop for BlockAlignment {
    fn name(&self) -> &'static str { "Layout/BlockAlignment" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["do_block", "block"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        let Some(end_kw) = shared::end_keyword(node)
            .or_else(|| shared::last_child_kind(node, "}")) else { return; };
        let start_col = shared::node_col(source, node);
        if shared::node_line(source, node) == shared::node_line(source, end_kw) { return; }
        if shared::node_col(source, end_kw) == start_col { return; }
        report::fix_indent(
            self, source, end_kw.start_byte(),
            format!("`end` is not aligned with `do` beginning at column {start_col}."),
            diagnostics, &mut corrections,
            shared::line_indent(source, end_kw.start_byte()), start_col,
        );
    }
}
