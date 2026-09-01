//! Layout/MultilineOperationIndentation.

mod check;
mod context;
mod indent;
mod line_scan;

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub(crate) use indent::aligned_method_call_col;

pub struct MultilineOperationIndentation;

impl Cop for MultilineOperationIndentation {
    fn name(&self) -> &'static str {
        "Layout/MultilineOperationIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["binary"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        check::check_binary(self, source, node, config, diagnostics, &mut corrections);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::run_cop_full_with_config;
    use std::collections::HashMap;

    crate::cop_fixture_tests!(
        MultilineOperationIndentation,
        "cops/layout/multiline_operation_indentation"
    );

    fn indented_config() -> CopConfig {
        CopConfig {
            options: HashMap::from([(
                "EnforcedStyle".into(),
                serde_yml::Value::String("indented".into()),
            )]),
            ..CopConfig::default()
        }
    }

    #[test]
    fn indented_if_or_chain_no_offense() {
        let diags = run_cop_full_with_config(
            &MultilineOperationIndentation,
            b"        if currency_id.blank? || !Currency.exists?(id: currency_id) ||\n            StringIdVersion.exists?(item_type: 'Currency')\n          stats[:skipped] += 1\n        end\n",
            indented_config(),
        );
        assert!(
            diags.is_empty(),
            "indented if-condition || continuation: {:?}",
            diags
        );
    }

    #[test]
    fn indented_return_unless_or_chain_no_offense() {
        let diags = run_cop_full_with_config(
            &MultilineOperationIndentation,
            b"      return unless ::Merchants::Firekassa::MERCHANT_TIDS.present? &&\n        merchant_tids.all? { |tid| withdraw_tids.include?(tid) }\n",
            indented_config(),
        );
        assert!(
            diags.is_empty(),
            "return unless || continuation: {:?}",
            diags
        );
    }

    #[test]
    fn indented_assignment_or_continuation_no_offense() {
        let diags = run_cop_full_with_config(
            &MultilineOperationIndentation,
            b"    wallet = Wallet.fee.find_by(blockchain_key: blockchain.real_key, tag: tag) ||\n      Wallet.deposit.find_by(blockchain_key: blockchain.real_key, tag: tag)\n",
            indented_config(),
        );
        assert!(
            diags.is_empty(),
            "indented assignment || continuation: {:?}",
            diags
        );
    }

    #[test]
    fn indented_assignment_plus_chain_no_offense() {
        let diags = run_cop_full_with_config(
            &MultilineOperationIndentation,
            b"    line = headers['TS-API-TIMESTAMP'].to_s +\n      headers['TS-API-API-KEY'].to_s +\n      payload\n",
            indented_config(),
        );
        assert!(
            diags.is_empty(),
            "indented assignment + chain: {:?}",
            diags
        );
    }
}
