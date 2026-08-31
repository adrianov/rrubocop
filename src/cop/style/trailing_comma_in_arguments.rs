//! Style/TrailingCommaInArguments.

use tree_sitter::Node;

use crate::cop::style::trailing_comma_args::{effective_locs, should_have_comma};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct TrailingCommaInArguments;

fn hanging_paren_list(source: &SourceFile, node: Node<'_>) -> Option<usize> {
    if node.start_position().row == node.end_position().row {
        return None;
    }
    let bytes = source.as_bytes();
    if bytes.get(node.start_byte()) != Some(&b'(') {
        return None;
    }
    let close = node.end_byte().saturating_sub(1);
    if bytes.get(close) != Some(&b')') {
        return None;
    }
    let (_, close_col) = source.offset_to_line_col(close);
    (crate::cop::shared::line_indent(source, close) == close_col).then_some(close)
}

fn arg_nodes(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cur = node.walk();
    // Tree-sitter places `heredoc_body` / `heredoc_end` as argument_list siblings;
    // RuboCop keeps them inside the heredoc argument node.
    node.named_children(&mut cur)
        .filter(|n| !matches!(n.kind(), "comment" | "heredoc_body" | "heredoc_end"))
        .collect()
}

fn contains_heredoc(node: Node<'_>) -> bool {
    if node.kind() == "heredoc_beginning" {
        return true;
    }
    let mut cur = node.walk();
    node.named_children(&mut cur).any(contains_heredoc)
}

fn args_have_heredoc(args: &[Node<'_>]) -> bool {
    args.iter().any(|a| contains_heredoc(*a))
}

/// RuboCop `comma_offset` — heredoc mode does not cross newlines.
fn trailing_comma_at(bytes: &[u8], last_end: usize, close: usize, heredoc: bool) -> Option<usize> {
    if last_end >= close || close > bytes.len() {
        return None;
    }
    let region = &bytes[last_end..close];
    if heredoc {
        comma_before_newline(region, last_end)
    } else {
        comma_after_ws(region, last_end)
    }
}

fn comma_before_newline(region: &[u8], base: usize) -> Option<usize> {
    for (i, &b) in region.iter().enumerate() {
        match b {
            b' ' | b'\t' => {}
            b',' => return Some(base + i),
            _ => return None,
        }
    }
    None
}

fn comma_after_ws(region: &[u8], base: usize) -> Option<usize> {
    let mut found = None;
    let mut in_comment = false;
    for (i, &b) in region.iter().enumerate() {
        if in_comment {
            if b == b'\n' {
                in_comment = false;
            }
            continue;
        }
        match b {
            b' ' | b'\t' | b'\n' | b'\r' => {}
            b'#' => in_comment = true,
            b',' if found.is_none() => found = Some(base + i),
            b',' => return None,
            _ => return None,
        }
    }
    found
}

fn report(
    cop: &TrailingCommaInArguments,
    source: &SourceFile,
    style: &str,
    has_comma: bool,
    want_comma: bool,
    at: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let msg = if want_comma && !has_comma {
        "Put a comma after the last parameter of a multiline method call."
    } else if !want_comma && has_comma {
        match style {
            "comma" => {
                "Avoid comma after the last parameter of a multiline method call, unless each item is on its own line."
            }
            "consistent_comma" => {
                "Avoid comma after the last parameter of a multiline method call, unless items are split onto multiple lines."
            }
            _ => "Avoid comma after the last parameter of a multiline method call.",
        }
    } else {
        return;
    };
    let (line, col) = source.offset_to_line_col(at);
    diagnostics.push(cop.diagnostic(source, line, col, msg.to_string()));
}

fn check_list(
    cop: &TrailingCommaInArguments,
    source: &SourceFile,
    args: &[Node<'_>],
    last: &Node<'_>,
    close: usize,
    style: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let comma_at = trailing_comma_at(
        source.as_bytes(),
        last.end_byte(),
        close,
        args_have_heredoc(args),
    );
    let locs = effective_locs(source, args);
    let want = should_have_comma(source, &locs, style, close);
    report(
        cop,
        source,
        style,
        comma_at.is_some(),
        want,
        comma_at.unwrap_or(last.end_byte()),
        diagnostics,
    );
}

impl Cop for TrailingCommaInArguments {
    fn name(&self) -> &'static str {
        "Style/TrailingCommaInArguments"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["argument_list"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(close) = hanging_paren_list(source, node) else {
            return;
        };
        let args = arg_nodes(node);
        let Some(last) = args.last() else {
            return;
        };
        let style = config.get_str("EnforcedStyleForMultiline", "no_comma");
        check_list(self, source, &args, last, close, style, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(TrailingCommaInArguments, "cops/style/trailing_comma_in_arguments");

    fn comma_cfg() -> CopConfig {
        let mut c = CopConfig::default();
        c.options.insert(
            "EnforcedStyleForMultiline".into(),
            serde_yml::Value::String("comma".into()),
        );
        c
    }

    #[test]
    fn comma_style_single_line_kwargs_ok() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &TrailingCommaInArguments,
            b"foo(\n  a: 1, b: 2,\n)\n",
            comma_cfg(),
        );
    }

    #[test]
    fn comma_style_shared_line_args_no_trailing_ok() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &TrailingCommaInArguments,
            b"foo(\n  1, 2, 3\n)\n",
            comma_cfg(),
        );
    }

    #[test]
    fn comma_style_heredoc_kwargs_trailing_ok() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &TrailingCommaInArguments,
            b"foo(\n  msg: <<~MESSAGE,\n    body\n  MESSAGE\n  level: :error,\n)\n",
            comma_cfg(),
        );
    }

    #[test]
    fn comma_style_heredoc_strip_trailing_ok() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &TrailingCommaInArguments,
            b"foo(\n  <<~MESSAGE.strip,\n    body\n  MESSAGE\n)\n",
            comma_cfg(),
        );
    }
}
