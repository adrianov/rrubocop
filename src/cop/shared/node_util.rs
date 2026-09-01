//! Node / source location helpers shared across cops.

use tree_sitter::Node;

use crate::parse::source::SourceFile;

/// Byte slice for a node.
pub fn node_bytes<'a>(source: &'a SourceFile, node: Node<'_>) -> &'a [u8] {
    &source.as_bytes()[node.start_byte()..node.end_byte()]
}

/// UTF-8 text for a node (lossy).
pub fn node_text(source: &SourceFile, node: Node<'_>) -> String {
    String::from_utf8_lossy(node_bytes(source, node)).into_owned()
}

/// Column of node start (0-based display column via SourceFile).
pub fn node_col(source: &SourceFile, node: Node<'_>) -> usize {
    source.offset_to_line_col(node.start_byte()).1
}

/// Line of node start (1-based).
pub fn node_line(source: &SourceFile, node: Node<'_>) -> usize {
    source.offset_to_line_col(node.start_byte()).0
}

/// Leading indent width of the line containing `offset` (spaces+tabs counted as 1 each).
pub fn line_indent(source: &SourceFile, offset: usize) -> usize {
    let (line, _) = source.offset_to_line_col(offset);
    let Some(start) = source.line_start(line) else {
        return 0;
    };
    let bytes = source.as_bytes();
    let mut i = start;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    i - start
}

/// True if the line at 1-based `line` is blank (or whitespace-only).
pub fn line_blank(source: &SourceFile, line: usize) -> bool {
    let Some(start) = source.line_start(line) else {
        return true;
    };
    let bytes = source.as_bytes();
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'\n' {
        if bytes[i] != b' ' && bytes[i] != b'\t' && bytes[i] != b'\r' {
            return false;
        }
        i += 1;
    }
    true
}

/// Find first direct child with kind `kind`.
pub fn child_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cur = node.walk();
    node.children(&mut cur).find(|c| c.kind() == kind)
}

/// Find last direct child with kind `kind`.
pub fn last_child_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cur = node.walk();
    node.children(&mut cur).filter(|c| c.kind() == kind).last()
}

/// Named children excluding punctuation.
pub fn named_kids<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = node.walk();
    node.named_children(&mut cur).collect()
}

/// Find anonymous `end` keyword token among children (or self range scan).
pub fn end_keyword<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.walk();
    node.children(&mut cur)
        .filter(|c| !c.is_named() && c.kind() == "end")
        .last()
}
