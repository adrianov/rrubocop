//! Block opener location and stabby-lambda brace detection.

use tree_sitter::Node;

use crate::parse::source::SourceFile;

pub(super) fn block_opener<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let kw = if node.kind() == "do_block" { "do" } else { "{" };
    let mut cur = node.walk();
    node.children(&mut cur)
        .find(|c| !c.is_named() && c.kind() == kw)
}

pub(super) fn brace_block_in_stabby_lambda(source: &SourceFile, opener: Node<'_>) -> bool {
    if opener.kind() != "{" {
        return false;
    }
    let Some(slice) = opener_line_bytes(source, opener) else {
        return false;
    };
    slice.windows(2).any(|w| w == b"->")
}

fn opener_line_bytes<'a>(source: &'a SourceFile, opener: Node<'_>) -> Option<&'a [u8]> {
    let line_no = source.offset_to_line_col(opener.start_byte()).0;
    let line_start = source.line_start(line_no)?;
    let line_end = source
        .lines()
        .nth(line_no.saturating_sub(1))
        .map(|l| line_start + l.len())
        .unwrap_or(opener.start_byte());
    let bytes = source.as_bytes();
    Some(&bytes[line_start..line_end.min(bytes.len())])
}

pub(super) fn first_non_ws_offset(source: &SourceFile, line: usize) -> Option<usize> {
    let start = source.line_start(line)?;
    let bytes = source.as_bytes();
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'\n' {
        if bytes[i] != b' ' && bytes[i] != b'\t' && bytes[i] != b'\r' {
            return Some(i);
        }
        i += 1;
    }
    None
}
