//! Layout/Layout/EmptyLinesAroundClassBody.

use tree_sitter::Node;

use crate::cop::layout::empty_body;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLinesAroundClassBody;

impl Cop for EmptyLinesAroundClassBody {
    fn name(&self) -> &'static str {
        "Layout/EmptyLinesAroundClassBody"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class", "singleton_class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let want_empty = config.get_str("EnforcedStyle", "no_empty_lines") == "empty_lines";
        empty_body::check_body(
            self,
            source,
            node,
            want_empty,
            "class",
            diagnostics,
            &mut corrections,
        );
    }
}
