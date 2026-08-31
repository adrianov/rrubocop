//! Layout/Layout/DefEndAlignment.

use tree_sitter::Node;

use crate::cop::layout::end_align;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct DefEndAlignment;

impl Cop for DefEndAlignment {
    fn name(&self) -> &'static str { "Layout/DefEndAlignment" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["method", "singleton_method"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyleAlignWith", "keyword");
        end_align::check_end(
            self, source, node, "def", style,
            diagnostics, &mut corrections,
        );
    }
}
