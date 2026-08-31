//! Style/NestedTernaryOperator
use tree_sitter::Node;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
pub struct NestedTernaryOperator;
impl Cop for NestedTernaryOperator {
    fn name(&self) -> &'static str { "Style/NestedTernaryOperator" }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["conditional"] }
    fn check_node(&self, source: &SourceFile, node: Node<'_>, _c: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, _corr: Option<&mut Vec<crate::correction::Correction>>) {
        let mut cur = node.walk();
        if !node.children(&mut cur).any(|ch| ch.kind() == "conditional") { return; }
        let (l, c) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, l, c,
            "Ternary operators must not be nested. Prefer `if` or `else` constructs instead.".into()));
    }
}
