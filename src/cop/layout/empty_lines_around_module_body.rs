//! Layout/Layout/EmptyLinesAroundModuleBody.

use tree_sitter::Node;

use crate::cop::layout::empty_body;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLinesAroundModuleBody;

impl Cop for EmptyLinesAroundModuleBody {
    fn name(&self) -> &'static str {
        "Layout/EmptyLinesAroundModuleBody"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["module"]
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
            "module",
            diagnostics,
            &mut corrections,
        );
    }
}
