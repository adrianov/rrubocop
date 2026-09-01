//! Style/TrailingCommaInHashLiteral.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::style::trailing_comma_args::skip_single_elem_inline_close;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct TrailingCommaInHashLiteral;

fn last_non_close<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.walk();
    node.children(&mut cur).collect::<Vec<_>>().into_iter().rev().find(|c| {
        node_bytes(source, *c) != b"}" && c.kind() != "comment"
    })
}

fn last_item(source: &SourceFile, node: Node<'_>) -> Option<(bool, usize)> {
    let last = last_non_close(source, node)?;
    if node_bytes(source, last) == b"," {
        Some((true, last.start_byte()))
    } else {
        // RuboCop `put_comma` locates on the last line of the last item (often `}`).
        Some((false, last.end_byte()))
    }
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
        if skip_single_elem_inline_close(source, node, b'}') {
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
