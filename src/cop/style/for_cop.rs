//! Style/For
use tree_sitter::Node;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
pub struct For;
impl Cop for For {
    fn name(&self) -> &'static str { "Style/For" }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["for"] }
    fn check_node(&self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, _corr: Option<&mut Vec<crate::correction::Correction>>) {
        if config.get_str("EnforcedStyle", "each") != "each" { return; }
        let (l, c) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, l, c, "Prefer `each` over `for`.".into()));
    }
}
