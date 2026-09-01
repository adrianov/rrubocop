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
        let want_percent = want_percent_r(source, node, b, style, allow_inner);
        report_regexp_style(self, source, node, is_slash, is_pct, want_percent, diagnostics);
    }
}

fn want_percent_r(
    source: &SourceFile,
    node: Node<'_>,
    bytes: &[u8],
    style: &str,
    allow_inner: bool,
) -> bool {
    let multiline = source.offset_to_line_col(node.start_byte()).0
        != source.offset_to_line_col(node.end_byte().saturating_sub(1)).0;
    let disallowed_slash = !allow_inner && regex_body(bytes).contains(&b'/');
    style == "percent_r"
        || (style == "mixed" && (multiline || disallowed_slash))
        || (style != "percent_r" && style != "mixed" && disallowed_slash)
}

fn report_regexp_style(
    cop: &RegexpLiteral,
    source: &SourceFile,
    node: Node<'_>,
    is_slash: bool,
    is_pct: bool,
    want_percent: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, col) = source.offset_to_line_col(node.start_byte());
    if is_slash && want_percent {
        diagnostics.push(cop.diagnostic(
            source,
            line,
            col,
            "Use `%r` around regular expression.".to_string(),
        ));
    } else if is_pct && !want_percent {
        diagnostics.push(cop.diagnostic(
            source,
            line,
            col,
            "Use `//` around regular expression.".to_string(),
        ));
    }
}

fn pct_r_body(rest: &[u8]) -> &[u8] {
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
        &inner[..end]
    } else {
        inner
    }
}

fn slash_regex_body(bytes: &[u8]) -> &[u8] {
    if !bytes.starts_with(b"/") || bytes.len() < 2 {
        return b"";
    }
    // Strip leading `/` and trailing `/flags`.
    let mut end = bytes.len() - 1;
    while end > 1 && bytes[end] != b'/' {
        end -= 1;
    }
    if end > 1 {
        &bytes[1..end]
    } else {
        b""
    }
}

fn regex_body(bytes: &[u8]) -> &[u8] {
    if let Some(rest) = bytes.strip_prefix(b"%r") {
        pct_r_body(rest)
    } else {
        slash_regex_body(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RegexpLiteral, "cops/style/regexp_literal");
}
