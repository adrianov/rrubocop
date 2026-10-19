//! Rails/SafeNavigationWithBlank — `if foo&.blank?` (any `if` node: modifier,
//! `unless`, `elsif`, ternary). Offense spans the whole conditional; the
//! unsafe autocorrect turns `&.` into `.`.

use tree_sitter::Node;

use crate::cop::shared::{node_bytes, push_replace, span_diagnostic};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SafeNavigationWithBlank;

const MSG: &str = "Avoid calling `blank?` with the safe navigation operator in conditionals.";

impl Cop for SafeNavigationWithBlank {
    fn name(&self) -> &'static str {
        "Rails/SafeNavigationWithBlank"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn safe_autocorrect(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &[
            "if",
            "unless",
            "elsif",
            "if_modifier",
            "unless_modifier",
            "conditional",
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(op) = node
            .child_by_field_name("condition")
            .and_then(|cond| safe_blank_operator(source, cond))
        else {
            return;
        };
        let mut diag = span_diagnostic(self, source, node.byte_range(), MSG.to_string());
        diag.corrected = push_replace(
            &mut corrections,
            op.start_byte(),
            op.end_byte(),
            ".",
            self.name(),
        );
        diagnostics.push(diag);
    }
}

/// The `&.` of a bare `recv&.blank?` call (RuboCop `(csend ... :blank?)`).
fn safe_blank_operator<'a>(source: &SourceFile, cond: Node<'a>) -> Option<Node<'a>> {
    if cond.kind() != "call"
        || cond.child_by_field_name("arguments").is_some()
        || cond.child_by_field_name("block").is_some()
    {
        return None;
    }
    let text = |n: Node<'_>| node_bytes(source, n);
    let op = cond.child_by_field_name("operator")?;
    (text(op) == b"&." && text(cond.child_by_field_name("method")?) == b"blank?").then_some(op)
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        SafeNavigationWithBlank,
        "cops/rails/safe_navigation_with_blank"
    );

    #[test]
    fn autocorrect_replaces_safe_navigation() {
        let fixed = crate::testutil::autocorrect_nodes(
            &SafeNavigationWithBlank,
            b"return unless foo&.blank?",
            CopConfig::default(),
        );
        assert_eq!(fixed, "return unless foo.blank?");
    }

    #[test]
    fn multiline_if_highlights_first_line_with_ellipsis() {
        let diags = crate::testutil::run_cop_full(
            &SafeNavigationWithBlank,
            b"if foo&.blank?\n  bar\nend\n",
        );
        assert_eq!(diags[0].source_line, "if foo&.blank? ...");
        assert_eq!(diags[0].highlight_length, 14);
    }
}
