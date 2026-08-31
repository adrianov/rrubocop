//! Comment / string / heredoc region map for source cops.

use tree_sitter::Node;

/// Byte ranges that are non-code (comments, strings, heredocs, regex).
#[derive(Debug, Default)]
pub struct CodeMap {
    ranges: Vec<(usize, usize)>,
}

impl CodeMap {
    pub fn from_tree(root: Node<'_>, src: &[u8]) -> Self {
        let mut ranges = Vec::new();
        collect_ranges(root, src, &mut ranges);
        ranges.sort_by_key(|r| r.0);
        Self { ranges }
    }

    pub fn covers(&self, offset: usize) -> bool {
        self.ranges
            .iter()
            .any(|&(s, e)| offset >= s && offset < e)
    }
}

fn collect_ranges(node: Node<'_>, src: &[u8], out: &mut Vec<(usize, usize)>) {
    let kind = node.kind();
    if matches!(
        kind,
        "comment"
            | "string"
            | "string_content"
            | "heredoc_body"
            | "heredoc_beginning"
            | "heredoc_end"
            | "regex"
            | "character"
            | "simple_symbol"
            | "hash_key_symbol"
            | "%w"
            | "%W"
            | "%i"
            | "%I"
            | "%q"
            | "%Q"
            | "%r"
            | "%s"
            | "%x"
    ) || kind.contains("string")
        || kind.contains("heredoc")
    {
        out.push((node.start_byte(), node.end_byte()));
        return;
    }
    // Still walk children for nested comments outside strings
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ranges(child, src, out);
    }
    let _ = src;
}
