//! Style/EndBlock
use tree_sitter::Node;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
pub struct EndBlock;
impl Cop for EndBlock {
    fn name(&self) -> &'static str { "Style/EndBlock" }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["end_block"] }
    fn check_node(&self, source: &SourceFile, node: Node<'_>, _c: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, _corr: Option<&mut Vec<crate::correction::Correction>>) {
        let (l, c) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, l, c,
            "Avoid the use of `END` blocks. Use `Kernel#at_exit` instead.".into()));
    }
}
