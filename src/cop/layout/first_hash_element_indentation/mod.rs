//! Layout/FirstHashElementIndentation — first key and `}` of a multiline `{` hash.

mod base;
mod report;

use tree_sitter::Node;

use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

use base::{braced, closing_brace, code_before, hash_pairs, indent_base, separator_offset};
use report::report_at;

pub struct FirstHashElementIndentation;

fn check_first_pair(
    cop: &dyn Cop,
    source: &SourceFile,
    hash: Node<'_>,
    config: &CopConfig,
    base_col: usize,
    kind: base::Base,
    width: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let pairs = hash_pairs(hash);
    let Some(first) = pairs.first().copied() else {
        return;
    };
    if shared::node_line(source, first) == shared::node_line(source, hash) {
        return;
    }
    report_at(
        cop,
        source,
        first,
        base_col + width + separator_offset(&pairs, config),
        format!(
            "Use {width} spaces for indentation in a hash, relative to {}.",
            kind.description()
        ),
        diagnostics,
        corrections,
    );
}

fn check_close(
    cop: &dyn Cop,
    source: &SourceFile,
    hash: Node<'_>,
    base_col: usize,
    kind: base::Base,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(close) = closing_brace(hash) else {
        return;
    };
    if code_before(source, close) {
        return;
    }
    report_at(
        cop,
        source,
        close,
        base_col,
        kind.brace_message().to_string(),
        diagnostics,
        corrections,
    );
}

fn check_hash(
    cop: &dyn Cop,
    source: &SourceFile,
    hash: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !braced(source, hash) {
        return;
    }
    let width = config.get_usize("IndentationWidth", 2);
    let (base_col, kind) = indent_base(source, hash, config);
    check_first_pair(
        cop,
        source,
        hash,
        config,
        base_col,
        kind,
        width,
        diagnostics,
        corrections,
    );
    check_close(cop, source, hash, base_col, kind, diagnostics, corrections);
}

impl Cop for FirstHashElementIndentation {
    fn name(&self) -> &'static str {
        "Layout/FirstHashElementIndentation"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["hash"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        check_hash(self, source, node, config, diagnostics, &mut corrections);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;

    crate::cop_fixture_tests!(
        FirstHashElementIndentation,
        "cops/layout/first_hash_element_indentation"
    );

    fn config_with(key: &str, value: &str) -> CopConfig {
        let mut config = CopConfig::default();
        config
            .options
            .insert(key.into(), serde_yml::Value::String(value.into()));
        config
    }

    #[test]
    fn align_braces_uses_opening_brace_column() {
        let cfg = config_with("EnforcedStyle", "align_braces");
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &FirstHashElementIndentation,
            b"a = {\n      a: 1\n    }\n",
            cfg.clone(),
        );
        crate::testutil::assert_cop_offenses_full_with_config(
            &FirstHashElementIndentation,
            b"var = {\n  a: 1\n  ^^^^ Layout/FirstHashElementIndentation: Use 2 spaces for indentation in a hash, relative to the position of the opening brace.\n}\n^ Layout/FirstHashElementIndentation: Indent the right brace the same as the left brace.\n",
            cfg,
        );
    }

    #[test]
    fn separator_style_indents_first_key_to_the_longest() {
        let cfg = config_with("EnforcedColonStyle", "separator");
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &FirstHashElementIndentation,
            b"a << {\n    a: 1,\n  aaa: 222\n}\n",
            cfg.clone(),
        );
        crate::testutil::assert_cop_offenses_full_with_config(
            &FirstHashElementIndentation,
            b"a << {\n       a: 1,\n       ^^^^ Layout/FirstHashElementIndentation: Use 2 spaces for indentation in a hash, relative to the start of the line where the left curly brace is.\n     aaa: 222\n}\n",
            cfg,
        );
    }

    #[test]
    fn fixed_argument_indentation_ignores_parentheses() {
        let mut cfg = CopConfig::default();
        cfg.options.insert(
            "FixedArgumentIndentation".into(),
            serde_yml::Value::Bool(true),
        );
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &FirstHashElementIndentation,
            b"func({\n  a: 1\n})\n",
            cfg,
        );
    }
}
