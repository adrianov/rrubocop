//! Style/InvertibleUnlessCondition — `unless x.any?` → `if x.none?`; the
//! offense spans the whole `unless` node (modifier form starts at its body).

use tree_sitter::Node;

use crate::cop::shared::{push_edits, span_diagnostic};
use crate::cop::style::heuristics::invert_unless;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct InvertibleUnlessCondition;

impl Cop for InvertibleUnlessCondition {
    fn name(&self) -> &'static str {
        "Style/InvertibleUnlessCondition"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn safe_autocorrect(&self) -> bool {
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
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some((current, preferred, edits)) = invert_unless(source, node, config) else {
            return;
        };
        let msg = format!("Prefer `if {preferred}` over `unless {current}`.");
        let mut diag = span_diagnostic(self, source, node.byte_range(), msg);
        diag.corrected = push_edits(&mut corrections, source, edits, self.name());
        diagnostics.push(diag);
    }
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
        config.options.insert(
            "InverseMethods".to_string(),
            serde_yml::from_str(yaml).expect("parse InverseMethods"),
        );
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
    fn autocorrect_matches_rubocop() {
        let src = b"foo unless !a && (b.any? || c != d)\nunless e.include? f\n  g\nend\nh unless not i\n";
        assert_eq!(
            crate::testutil::autocorrect_nodes(
                &InvertibleUnlessCondition,
                src,
                config_with_inverse_methods(&merged_inverse_methods_yaml()),
            ),
            // RuboCop removes only the `not` selector, leaving a double space.
            "foo if a || (b.none? && c == d)\nif e.exclude? f\n  g\nend\nh if  i\n"
        );
    }

    #[test]
    fn project_defined_inverse_methods_apply() {
        let source = concat!(
            "foo unless x.writable?\n",
            "^^^^^^^^^^^^^^^^^^^^^^ Style/InvertibleUnlessCondition: Prefer `if x.readonly?` over `unless x.writable?`.\n",
        );
        crate::testutil::assert_cop_offenses_full_with_config(
            &InvertibleUnlessCondition,
            source.as_bytes(),
            config_with_inverse_methods(r#":writable?: :readonly?"#),
        );
    }
}
