//! Detect `end rescue` modifier after a block closer.

use tree_sitter::Node;

use crate::parse::source::SourceFile;

pub(super) fn closer_follows_rescue_modifier(source: &SourceFile, closer: Node<'_>) -> bool {
    let bytes = source.as_bytes();
    let pos = skip_hspace(bytes, closer.end_byte());
    is_rescue_word(bytes, pos)
}

fn skip_hspace(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && matches!(bytes[pos], b' ' | b'\t' | b'\r') {
        pos += 1;
    }
    pos
}

fn is_rescue_word(bytes: &[u8], pos: usize) -> bool {
    pos + 6 <= bytes.len()
        && &bytes[pos..pos + 6] == b"rescue"
        && word_boundary_before(bytes, pos)
        && word_boundary_after(bytes, pos + 6)
}

fn word_boundary_before(bytes: &[u8], pos: usize) -> bool {
    pos == 0 || (!bytes[pos - 1].is_ascii_alphanumeric() && bytes[pos - 1] != b'_')
}

fn word_boundary_after(bytes: &[u8], end: usize) -> bool {
    end >= bytes.len() || (!bytes[end].is_ascii_alphanumeric() && bytes[end] != b'_')
}
