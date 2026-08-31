//! Style/QuotedSymbols.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct QuotedSymbols;

impl Cop for QuotedSymbols {
    fn name(&self) -> &'static str {
        "Style/QuotedSymbols"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["simple_symbol", "symbol", "hash_key_symbol"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "same_as_string_literals");
        if style != "same_as_string_literals" {
            return;
        }
        if !is_plain_quoted(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Prefer unquoted symbols when possible.".to_string(),
        ));
    }
}

fn is_plain_quoted(source: &SourceFile, node: Node<'_>) -> bool {
    let b = node_bytes(source, node);
    if !(b.starts_with(b":\"") || b.starts_with(b":'")) {
        return false;
    }
    let inner = &b[2..b.len().saturating_sub(1)];
    inner
        .iter()
        .all(|c| c.is_ascii_alphanumeric() || *c == b'_')
}
