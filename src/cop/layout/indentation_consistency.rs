//! Layout/IndentationConsistency.
//!
//! RuboCop only checks begin/kwbegin statement lists. In tree-sitter those are
//! `body_statement`, `begin`, `then`, `else`, `do`, `ensure`, `block_body`,
//! `parenthesized_statements`, `program`, and `begin_block` — not class/module/
//! method nodes (whose named children include names, `self`, parameters, etc.).

use tree_sitter::Tree;

use crate::cop::layout::indentation_consistency_check;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct IndentationConsistency;

impl Cop for IndentationConsistency {
    fn name(&self) -> &'static str {
        "Layout/IndentationConsistency"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        _code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let indented = config.get_str("EnforcedStyle", "normal") == "indented_internal_methods";
        shared::for_each_descendant(tree.root_node(), |n| {
            indentation_consistency_check::check_list(
                self,
                source,
                n,
                indented,
                diagnostics,
                &mut corrections,
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(IndentationConsistency, "cops/layout/indentation_consistency");
}
