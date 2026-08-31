//! Style/TrailingCommaInHashLiteral.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct TrailingCommaInHashLiteral;

impl Cop for TrailingCommaInHashLiteral {
    fn name(&self) -> &'static str {
        "Style/TrailingCommaInHashLiteral"
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
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.start_position().row == node.end_position().row {
            return;
        }
        let Some((has_comma, start)) = last_item(source, node) else {
            return;
        };
        report(
            self,
            source,
            config.get_str("EnforcedStyleForMultiline", "no_comma"),
            has_comma,
            start,
            diagnostics,
        );
    }
}

fn last_item(source: &SourceFile, node: Node<'_>) -> Option<(bool, usize)> {
    let mut cur = node.walk();
    let children: Vec<_> = node.children(&mut cur).collect();
    let last = children.iter().rev().find(|c| {
        let t = node_bytes(source, **c);
        t != b"}" && c.kind() != "comment"
    })?;
    Some((node_bytes(source, *last) == b",", last.start_byte()))
}

fn report(
    cop: &TrailingCommaInHashLiteral,
    source: &SourceFile,
    style: &str,
    has_comma: bool,
    start: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, col) = source.offset_to_line_col(start);
    let msg = match style {
        "comma" | "consistent_comma" if !has_comma => {
            "Put a comma after the last item of a multiline hash."
        }
        "no_comma" if has_comma => "Avoid comma after the last item of a multiline hash.",
        _ => return,
    };
    diagnostics.push(cop.diagnostic(source, line, col, msg.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;
    use crate::testutil::assert_cop_no_offenses_full_with_config;

    crate::cop_fixture_tests!(TrailingCommaInHashLiteral, "cops/style/trailing_comma_in_hash_literal");

    #[test]
    fn comma_before_trailing_comment_consistent() {
        let mut config = CopConfig::default();
        config.options.insert(
            "EnforcedStyleForMultiline".into(),
            serde_yml::Value::String("consistent_comma".into()),
        );
        assert_cop_no_offenses_full_with_config(
            &TrailingCommaInHashLiteral,
            b"{\n  a: 1,\n  b: 2, # comment\n}\n",
            config,
        );
    }
}
