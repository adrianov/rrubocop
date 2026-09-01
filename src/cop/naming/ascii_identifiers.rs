//! Naming/AsciiIdentifiers — non-ASCII in identifiers.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct AsciiIdentifiers;

/// RuboCop only checks `tIDENTIFIER` / `tCONSTANT`, not symbol tokens (`:НЕИЗВЕСТНО`).
fn symbol_ident(source: &SourceFile, node: Node<'_>) -> bool {
    let start = node.start_byte();
    let bytes = source.as_bytes();
    (start > 0 && bytes[start - 1] == b':') || unicode_symbol_tail(bytes, start)
}

fn unicode_symbol_tail(bytes: &[u8], start: usize) -> bool {
    let Ok(s) = std::str::from_utf8(&bytes[..start]) else {
        return false;
    };
    let mut chars = s.chars().rev();
    let Some(prev) = chars.next() else {
        return false;
    };
    is_unicode_ident(prev) && chars.take_while(|c| is_ident_char(*c) || *c == ':').any(|c| c == ':')
}

fn is_unicode_ident(c: char) -> bool {
    (c.is_alphanumeric() || c == '_') && !c.is_ascii()
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Cop for AsciiIdentifiers {
    fn name(&self) -> &'static str {
        "Naming/AsciiIdentifiers"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &[
            "identifier",
            "constant",
            "instance_variable",
            "class_variable",
            "global_variable",
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let bytes = &source.as_bytes()[node.start_byte()..node.end_byte()];
        if bytes.iter().all(|&b| b.is_ascii()) || symbol_ident(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Use only ascii characters in identifiers.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(AsciiIdentifiers, "cops/naming/ascii_identifiers");
}
