//! Style/BeginBlock
use tree_sitter::Node;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
pub struct BeginBlock;
impl Cop for BeginBlock {
    fn name(&self) -> &'static str { "Style/BeginBlock" }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["begin_block"] }
    fn check_node(&self, source: &SourceFile, node: Node<'_>, _c: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, _corr: Option<&mut Vec<crate::correction::Correction>>) {
        let (l, c) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, l, c, "Avoid the use of `BEGIN` blocks.".into()));
    }
}
