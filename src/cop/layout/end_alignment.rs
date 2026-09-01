//! Layout/EndAlignment.

use tree_sitter::Node;

use crate::cop::layout::end_align;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EndAlignment;

fn base_name(kind: &str) -> &'static str {
    match kind {
        "class" => "class",
        "module" => "module",
        "if" => "if",
        "unless" => "unless",
        "while" => "while",
        "until" => "until",
        "case" => "case",
        _ => "def",
    }
}

impl Cop for EndAlignment {
    fn name(&self) -> &'static str {
        "Layout/EndAlignment"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        // `begin`/`end` is Layout/BeginEndAlignment; `do`/`end` is BlockAlignment.
        // RuboCop EndAlignment has no `on_kwbegin`.
        &[
            "class",
            "module",
            "if",
            "unless",
            "while",
            "until",
            "case",
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyleAlignWith", "keyword");
        end_align::check_end(
            self,
            source,
            node,
            base_name(node.kind()),
            style,
            diagnostics,
            &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::run_cop_full_with_config;
    use std::collections::HashMap;

    crate::cop_fixture_tests!(EndAlignment, "cops/layout/end_alignment");

    fn variable_config() -> CopConfig {
        CopConfig {
            options: HashMap::from([(
                "EnforcedStyleAlignWith".into(),
                serde_yml::Value::String("variable".into()),
            )]),
            ..CopConfig::default()
        }
    }

    #[test]
    fn variable_style_operator_assignment_if_no_offense() {
        let src = b"      trades.each do |t|\n        total_from += if cr.from_currency_id == t.currency_id\n          t.volume\n        else\n          t.funds\n        end\n      end\n";
        let diags = run_cop_full_with_config(&EndAlignment, src, variable_config());
        assert!(
            diags.is_empty(),
            "variable style should align end with += assignment: {:?}",
            diags
        );
    }

    #[test]
    fn variable_style_shovel_if_no_offense() {
        let src = b"      warnings << if initiator\n        t('msg')\n      else\n        t('other')\n      end\n";
        let diags = run_cop_full_with_config(&EndAlignment, src, variable_config());
        assert!(
            diags.is_empty(),
            "variable style should align end with << receiver: {:?}",
            diags
        );
    }
}
