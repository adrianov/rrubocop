//! Style/RegexpLiteral.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RegexpLiteral;

impl Cop for RegexpLiteral {
    fn name(&self) -> &'static str {
        "Style/RegexpLiteral"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["regex"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "slashes");
        let allow_inner = config.get_bool("AllowInnerSlashes", false);
        let b = node_bytes(source, node);
        let is_pct = b.starts_with(b"%r");
        let is_slash = b.starts_with(b"/");
        if !is_pct && !is_slash {
            return;
        }
        let multiline = source.offset_to_line_col(node.start_byte()).0
            != source.offset_to_line_col(node.end_byte().saturating_sub(1)).0;
        let has_inner_slash = regex_body(b).contains(&b'/');
        let disallowed_slash = !allow_inner && has_inner_slash;

        let want_percent = style == "percent_r"
            || (style == "mixed" && (multiline || disallowed_slash))
            || (style != "percent_r" && style != "mixed" && disallowed_slash);

        let (line, col) = source.offset_to_line_col(node.start_byte());
        if is_slash && want_percent {
            diagnostics.push(self.diagnostic(
                source,
                line,
                col,
                "Use `%r` around regular expression.".to_string(),
            ));
        } else if is_pct && !want_percent {
            diagnostics.push(self.diagnostic(
                source,
                line,
                col,
                "Use `//` around regular expression.".to_string(),
            ));
        }
    }
}

fn regex_body(bytes: &[u8]) -> &[u8] {
    if let Some(rest) = bytes.strip_prefix(b"%r") {
        if rest.is_empty() {
            return rest;
        }
        let open = rest[0];
        let close = match open {
            b'(' => b')',
            b'[' => b']',
            b'{' => b'}',
            b'<' => b'>',
            c => c,
        };
        let inner = &rest[1..];
        if let Some(end) = inner.iter().rposition(|&b| b == close) {
            return &inner[..end];
        }
        return inner;
    }
    if bytes.starts_with(b"/") && bytes.len() >= 2 {
        // Strip leading `/` and trailing `/flags`.
        let mut end = bytes.len() - 1;
        while end > 1 && bytes[end] != b'/' {
            end -= 1;
        }
        if end > 1 {
            return &bytes[1..end];
        }
    }
    b""
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RegexpLiteral, "cops/style/regexp_literal");
}
