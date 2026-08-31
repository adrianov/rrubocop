//! Style/RescueModifier
use tree_sitter::Node;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
pub struct RescueModifier;
impl Cop for RescueModifier {
    fn name(&self) -> &'static str { "Style/RescueModifier" }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["rescue_modifier"] }
    fn check_node(&self, source: &SourceFile, node: Node<'_>, _c: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, _corr: Option<&mut Vec<crate::correction::Correction>>) {
        let (l, c) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, l, c, "Avoid using `rescue` in its modifier form.".into()));
    }
}
