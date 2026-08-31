//! Layout/SpaceAroundKeyword.

use tree_sitter::Tree;

use crate::cop::layout::keyword_space;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct SpaceAroundKeyword;

impl Cop for SpaceAroundKeyword {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundKeyword"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = (code_map, config);
        let bytes = source.as_bytes();
        shared::for_each_descendant(tree.root_node(), |n| {
            if !keyword_space::should_check(bytes, n) {
                return;
            }
            let Some((kw_start, end)) = keyword_space::kw_span(bytes, n) else {
                return;
            };
            keyword_space::check_after(
                self,
                source,
                bytes,
                n.kind(),
                kw_start,
                end,
                diagnostics,
                &mut corrections,
            );
            keyword_space::check_before(
                self,
                source,
                bytes,
                kw_start,
                end,
                diagnostics,
                &mut corrections,
            );
        });
    }
}
