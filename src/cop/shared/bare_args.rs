//! Parentheses around a bare argument list when `&block` becomes `&`.

use tree_sitter::Node;

use crate::correction::Correction;
use crate::parse::source::SourceFile;

use super::push_replace;

/// Replace `node` and wrap a bare `foo &block` as `foo(&)`.
pub fn replace_arg(
    source: &SourceFile,
    node: Node<'_>,
    text: &str,
    corrections: &mut Option<&mut Vec<Correction>>,
    cop_name: &'static str,
) -> bool {
    if corrections.is_none() {
        return false;
    }
    wrap_bare(source, node, corrections, cop_name);
    push_replace(
        corrections,
        node.start_byte(),
        node.end_byte(),
        text,
        cop_name,
    )
}

fn wrap_bare(
    source: &SourceFile,
    node: Node<'_>,
    corrections: &mut Option<&mut Vec<Correction>>,
    cop_name: &'static str,
) {
    let Some(list) = bare_list(source, node) else {
        return;
    };
    push_replace(
        corrections,
        trim_space(source, list.start_byte()),
        list.start_byte(),
        "(",
        cop_name,
    );
    push_replace(corrections, list.end_byte(), list.end_byte(), ")", cop_name);
}

fn bare_list<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    let list = node.parent()?;
    if !matches!(list.kind(), "argument_list" | "method_parameters") {
        return None;
    }
    (source.as_bytes().get(list.start_byte()) != Some(&b'(')).then_some(list)
}

fn trim_space(source: &SourceFile, at: usize) -> usize {
    let bytes = source.as_bytes();
    let mut open = at;
    while open > 0 && bytes[open - 1].is_ascii_whitespace() {
        open -= 1;
    }
    open
}
