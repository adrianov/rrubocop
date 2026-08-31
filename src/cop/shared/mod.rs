//! Shared tree-sitter helpers for cops (nitrocop-inspired, AST-adapted).

use tree_sitter::Node;

use crate::correction::Correction;
use crate::parse::source::SourceFile;

/// Method/name field of a call-like node.
pub fn method_node(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("method")
        .or_else(|| node.child_by_field_name("name"))
}

/// Push a byte-range replacement when collecting corrections.
pub fn push_replace(
    corrections: &mut Option<&mut Vec<Correction>>,
    start: usize,
    end: usize,
    replacement: impl Into<String>,
    cop_name: &'static str,
) -> bool {
    let Some(corr) = corrections.as_deref_mut() else {
        return false;
    };
    corr.push(Correction {
        start,
        end,
        replacement: replacement.into(),
        cop_name,
        cop_index: 0,
    });
    true
}

/// Byte slice for a node.
pub fn node_bytes<'a>(source: &'a SourceFile, node: Node<'_>) -> &'a [u8] {
    &source.as_bytes()[node.start_byte()..node.end_byte()]
}

/// UTF-8 text for a node (lossy).
pub fn node_text(source: &SourceFile, node: Node<'_>) -> String {
    String::from_utf8_lossy(node_bytes(source, node)).into_owned()
}

/// True if `name` is SCREAMING_SNAKE_CASE (RuboCop Naming/ConstantName).
pub fn is_screaming_snake_case(name: &[u8]) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut has_letter = false;
    for &b in name {
        match b {
            b'A'..=b'Z' => has_letter = true,
            b'0'..=b'9' | b'_' => {}
            _ => return false,
        }
    }
    has_letter
}

/// Method name of a `call` / `command` / `method` node, if any.
pub fn call_method_name<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    let meth = node
        .child_by_field_name("method")
        .or_else(|| node.child_by_field_name("name"))?;
    Some(node_bytes(source, meth))
}

/// Receiver of a call-like node.
pub fn call_receiver(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("receiver")
}

/// True if node is a constant read whose text equals `name` (e.g. `JSON`).
pub fn is_const_named(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    match node.kind() {
        "constant" => node_bytes(source, node) == name,
        "scope_resolution" => node
            .child_by_field_name("name")
            .map(|n| node_bytes(source, n) == name)
            .unwrap_or(false),
        _ => false,
    }
}

/// Direct named children (field `arguments` or positional).
pub fn argument_nodes<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    if let Some(args) = node.child_by_field_name("arguments") {
        let mut cur = args.walk();
        return args
            .named_children(&mut cur)
            .filter(|n| n.kind() != ",")
            .collect();
    }
    Vec::new()
}

/// Walk all descendants, invoking `f` on each.
pub fn for_each_descendant(node: Node<'_>, mut f: impl FnMut(Node<'_>)) {
    fn walk(node: Node<'_>, f: &mut impl FnMut(Node<'_>)) {
        f(node);
        let mut cur = node.walk();
        for child in node.children(&mut cur) {
            walk(child, f);
        }
    }
    walk(node, &mut f);
}


/// Collect all `comment` nodes under `root`.
pub fn collect_comments<'a>(root: Node<'a>) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    fn walk<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
        if node.kind() == "comment" {
            out.push(node);
        }
        let mut cur = node.walk();
        for child in node.children(&mut cur) {
            walk(child, out);
        }
    }
    walk(root, &mut out);
    out
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

/// True if bytes in [start,end) are only spaces/tabs.
pub fn only_spaces(bytes: &[u8], start: usize, end: usize) -> bool {
    if end <= start {
        return true;
    }
    bytes[start..end.min(bytes.len())]
        .iter()
        .all(|&b| b == b' ' || b == b'\t')
}

/// True if [start,end) is non-empty and contains only spaces/tabs (no newlines).
pub fn only_hspace(bytes: &[u8], start: usize, end: usize) -> bool {
    if end <= start {
        return false;
    }
    bytes[start..end.min(bytes.len())]
        .iter()
        .all(|&b| b == b' ' || b == b'\t')
}

/// True if [start,end) contains a space or tab.
pub fn has_hspace(bytes: &[u8], start: usize, end: usize) -> bool {
    if end <= start {
        return false;
    }
    bytes[start..end.min(bytes.len())]
        .iter()
        .any(|&b| b == b' ' || b == b'\t')
}

/// Find anonymous `end` keyword token among children (or self range scan).
pub fn end_keyword<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cur = node.walk();
    node.children(&mut cur)
        .filter(|c| !c.is_named() && c.kind() == "end")
        .last()
}
