//! Layout/RescueEnsureAlignment.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RescueEnsureAlignment;

impl Cop for RescueEnsureAlignment {
    fn name(&self) -> &'static str { "Layout/RescueEnsureAlignment" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["rescue", "ensure"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        let Some(parent) = node.parent() else { return; };
        let base_col = shared::node_col(source, parent);
        if shared::node_col(source, node) == base_col { return; }
        let (kl, kc) = source.offset_to_line_col(node.start_byte());
        let kw = node.kind();
        report::fix_indent(
            self, source, node.start_byte(),
            format!("`{kw}` at {kl}, {kc} is not aligned with beginning at column {base_col}."),
            diagnostics, &mut corrections,
            shared::line_indent(source, node.start_byte()), base_col,
        );
    }
}
