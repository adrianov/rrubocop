//! Layout/Layout/ArgumentAlignment.

use tree_sitter::Node;

use crate::cop::layout::align_items;
use crate::cop::layout::first_indent;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ArgumentAlignment;

impl Cop for ArgumentAlignment {
    fn name(&self) -> &'static str { "Layout/ArgumentAlignment" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["argument_list"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if !first_indent::argument_list_opens_with_paren(node) {
            return;
        }
        let style = config.get_str("EnforcedStyle", "with_first_argument");
        let width = config.get_usize("IndentationWidth", 2);
        align_items::check_align(
            self, source, node, style, width,
            "Align the arguments of a method call if they span more than one line.",
            diagnostics, &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ArgumentAlignment, "cops/layout/argument_alignment");

    #[test]
    fn no_offense_command_fixture() {
        crate::testutil::assert_cop_no_offenses_full(
            &ArgumentAlignment,
            include_bytes!("../../../tests/fixtures/cops/layout/argument_alignment/no_offense_command.rb"),
        );
    }

    #[test]
    fn no_offense_kwargs_fixture() {
        crate::testutil::assert_cop_no_offenses_full(
            &ArgumentAlignment,
            include_bytes!("../../../tests/fixtures/cops/layout/argument_alignment/no_offense_kwargs.rb"),
        );
    }

    #[test]
    fn no_offense_trailing_kwargs_fixture() {
        crate::testutil::assert_cop_no_offenses_full(
            &ArgumentAlignment,
            include_bytes!("../../../tests/fixtures/cops/layout/argument_alignment/no_offense_trailing_kwargs.rb"),
        );
    }
}
