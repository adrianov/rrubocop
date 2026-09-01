//! Anonymous block-arg ERROR (`foo(&)` / `foo(&,)`) that MRI accepts (Ruby 3.1+).

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

/// Only suppress ERROR `&` when the next non-whitespace token is `,` or `)` and the
/// node sits in an argument list (not e.g. `& 1` / `&Foo`).
pub(super) fn anonymous_block_arg_error(source: &SourceFile, node: Node<'_>) -> bool {
    if !node.is_error() || node_bytes(source, node) != b"&" {
        return false;
    }
    if !inside_argument_list(node) {
        return false;
    }
    let bytes = source.as_bytes();
    let mut i = node.end_byte();
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r') {
        i += 1;
    }
    matches!(bytes.get(i), Some(b')' | b','))
}

fn inside_argument_list(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(n) = p {
        if matches!(
            n.kind(),
            "argument_list"
                | "command_argument_list"
                | "method_parameters"
                | "parameters"
                | "formal_parameters"
        ) {
            return true;
        }
        if matches!(
            n.kind(),
            "program" | "method" | "singleton_method" | "class" | "module"
        ) {
            break;
        }
        p = n.parent();
    }
    false
}
