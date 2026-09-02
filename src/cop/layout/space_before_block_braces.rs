//! Layout/SpaceBeforeBlockBraces.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceBeforeBlockBraces;

fn find_lbrace<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.walk();
    node.children(&mut cur).find(|c| c.kind() == "{")
}

fn find_rbrace<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.walk();
    let kids: Vec<_> = node.children(&mut cur).collect();
    kids.iter().rev().find(|c| c.kind() == "}").copied()
}

fn ws_before(bytes: &[u8], start: usize) -> usize {
    let mut ws = start;
    while ws > 0 && matches!(bytes[ws - 1], b' ' | b'\t') {
        ws -= 1;
    }
    ws
}

fn empty_braces(_bytes: &[u8], lbrace: Node<'_>, rbrace: Node<'_>) -> bool {
    lbrace.end_byte() == rbrace.start_byte()
}

fn enforced_style(config: &CopConfig, empty: bool) -> bool {
    if empty {
        match config.options.get("EnforcedStyleForEmptyBraces") {
            Some(v) => v.as_str() == Some("space"),
            None => config.get_str("EnforcedStyle", "space") != "no_space",
        }
    } else {
        config.get_str("EnforcedStyle", "space") != "no_space"
    }
}

fn report_spacing(
    cop: &SpaceBeforeBlockBraces,
    source: &SourceFile,
    want: bool,
    has_space: bool,
    start: usize,
    ws_start: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if want && !has_space {
        report::insert_space(
            cop,
            source,
            start,
            "Space missing to the left of {.".into(),
            diagnostics,
            corrections,
            start,
        );
    } else if !want && has_space {
        report::report_fix(
            cop,
            source,
            ws_start,
            "Space detected to the left of {.".into(),
            diagnostics,
            corrections,
            ws_start,
            start,
            String::new(),
        );
    }
}

impl Cop for SpaceBeforeBlockBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeBlockBraces"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["block"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let bytes = source.as_bytes();
        let Some(lbrace) = find_lbrace(node) else {
            return;
        };
        let start = lbrace.start_byte();
        if start == 0 {
            return;
        }
        let empty = find_rbrace(node).is_some_and(|r| empty_braces(bytes, lbrace, r));
        let want = enforced_style(config, empty);
        let ws_start = ws_before(bytes, start);
        report_spacing(
            self,
            source,
            want,
            ws_start < start,
            start,
            ws_start,
            diagnostics,
            &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;
    use std::collections::HashMap;

    crate::cop_fixture_tests!(SpaceBeforeBlockBraces, "cops/layout/space_before_block_braces");

    #[test]
    fn empty_stabby_lambda_no_space_before_brace() {
        let config = CopConfig {
            options: HashMap::from([
                (
                    "EnforcedStyleForEmptyBraces".into(),
                    serde_yml::Value::String("no_space".into()),
                ),
                (
                    "EnforcedStyle".into(),
                    serde_yml::Value::String("space".into()),
                ),
            ]),
            ..CopConfig::default()
        };
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &SpaceBeforeBlockBraces,
            b"::Rack::Handler::Puma.config(->{}, @options)\n",
            config.clone(),
        );
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &SpaceBeforeBlockBraces,
            b"block = proc { } unless block\n",
            config,
        );
    }
}
