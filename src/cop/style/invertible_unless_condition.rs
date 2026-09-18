//! Style/InvertibleUnlessCondition — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_invertible_unless_condition;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct InvertibleUnlessCondition;

impl Cop for InvertibleUnlessCondition {
    fn name(&self) -> &'static str {
        "Style/InvertibleUnlessCondition"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["unless", "unless_modifier"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_invertible_unless_condition(source, node, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(unless_keyword_start(node).unwrap_or(node.start_byte()));
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Style/InvertibleUnlessCondition offense.".to_string(),
        ));
    }
}

/// The `unless` keyword's byte offset. Modifier nodes start at the whole
/// statement (`foo unless x`), so the offense must anchor to the token itself.
fn unless_keyword_start(node: Node<'_>) -> Option<usize> {
    node.children(&mut node.walk())
        .find(|child| child.kind() == "unless" && !child.is_named())
        .map(|keyword| keyword.start_byte())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RuboCop core default.yml pairs (`Style/InvertibleUnlessCondition`).
    const CORE_INVERSE_METHODS: &str = ":!=: :==
:>: :<=
:<=: :>
:<: :>=
:>=: :<
:!~: :=~
:zero?: :nonzero?
:nonzero?: :zero?
:any?: :none?
:none?: :any?
:even?: :odd?
:odd?: :even?
";

    /// Core pairs plus the Active Support additions rubocop-rails merges in —
    /// the resolved config CI's `bundle exec rubocop` runs peatio with.
    fn merged_inverse_methods_yaml() -> String {
        let mut yaml = CORE_INVERSE_METHODS.to_string();
        yaml.push_str(
            ":present?: :blank?\n:blank?: :present?\n:include?: :exclude?\n:exclude?: :include?\n",
        );
        yaml
    }

    fn config_with_inverse_methods(yaml: &str) -> CopConfig {
        let mut config = CopConfig::default();
        let value: serde_yml::Value = serde_yml::from_str(yaml).expect("parse InverseMethods");
        config.options.insert("InverseMethods".to_string(), value);
        config
    }

    #[test]
    fn offense_fixture() {
        crate::testutil::assert_cop_offenses_full_with_config(
            &InvertibleUnlessCondition,
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/cops/style/invertible_unless_condition/offense.rb"
            )),
            config_with_inverse_methods(&merged_inverse_methods_yaml()),
        );
    }

    #[test]
    fn no_offense_fixture() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &InvertibleUnlessCondition,
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/cops/style/invertible_unless_condition/no_offense.rb"
            )),
            config_with_inverse_methods(&merged_inverse_methods_yaml()),
        );
    }

    #[test]
    fn include_is_not_invertible_without_active_support_methods() {
        // Plain rubocop (no rubocop-rails plugin) has no `include?` inverse,
        // so `unless x.include?(y)` must stay clean.
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &InvertibleUnlessCondition,
            b"foo unless x.include?(y)\n",
            config_with_inverse_methods(CORE_INVERSE_METHODS),
        );
    }

    #[test]
    fn project_defined_inverse_methods_apply() {
        let source = concat!(
            "foo unless x.writable?\n",
            "    ^^^^^ Style/InvertibleUnlessCondition: Style/InvertibleUnlessCondition offense.\n",
        );
        crate::testutil::assert_cop_offenses_full_with_config(
            &InvertibleUnlessCondition,
            source.as_bytes(),
            config_with_inverse_methods(r#":writable?: :readonly?"#),
        );
    }
}
