//! Shared tree-sitter helpers for cops (nitrocop-inspired, AST-adapted).

mod bare_args;
mod block_pass;
mod node_util;

pub use bare_args::replace_arg;
pub use block_pass::{EXCLUDED_BLOCK_PARENTS, METHOD_STOPS, NESTED_SCOPES, Passes, ScanOpts, scan};

pub use node_util::{
    child_kind, end_keyword, field_text, last_child_kind, line_blank, line_indent, named_kids,
    node_bytes, node_col, node_line, node_text,
};

use std::ops::Range;

use tree_sitter::Node;

use crate::cop::Cop;
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
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

/// `(start, end, replacement)` byte edit.
pub type Edit = (usize, usize, String);

/// Push one offense's non-overlapping edits as a single replacement,
/// so an overlap drops all of them (RuboCop applies a cop's edits atomically).
pub fn push_edits(
    corrections: &mut Option<&mut Vec<Correction>>,
    source: &SourceFile,
    mut edits: Vec<Edit>,
    cop_name: &'static str,
) -> bool {
    edits.sort_by_key(|e| e.0);
    let (Some(&(start, ..)), Some(&(_, end, _))) = (edits.first(), edits.last()) else {
        return false;
    };
    let mut text = source.as_bytes()[start..end].to_vec();
    for (s, e, replacement) in edits.into_iter().rev() {
        text.splice(s - start..e - start, replacement.into_bytes());
    }
    push_replace(
        corrections,
        start,
        end,
        String::from_utf8_lossy(&text),
        cop_name,
    )
}

/// Offense spanning byte `range`, RuboCop clang style: located at its start and
/// underlined to its first line end; a multi-line range shows its first line plus ` ...`.
pub fn span_diagnostic(
    cop: &dyn Cop,
    source: &SourceFile,
    range: Range<usize>,
    message: String,
) -> Diagnostic {
    let (line, col) = source.offset_to_line_col(range.start);
    let mut diag = cop.diagnostic(source, line, col, message);
    let text = String::from_utf8_lossy(&source.as_bytes()[range]);
    diag.highlight_length = text.lines().next().map_or(1, |l| l.chars().count());
    if text.contains('\n')
        && let Some(line) = source.line_text(line)
    {
        diag.source_line = format!("{line} ...");
    }
    diag
}

/// True if `name` is SCREAMING_SNAKE_CASE (RuboCop Naming/ConstantName).
/// Allows Unicode uppercase letters (e.g. `KIND_НАЧИСЛЕНИЕ`).
pub fn is_screaming_snake_case(name: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(name) else {
        return false;
    };
    if s.is_empty() {
        return false;
    }
    let mut has_letter = false;
    for c in s.chars() {
        if c.is_uppercase() {
            has_letter = true;
        } else if c.is_ascii_digit() || c == '_' {
            // ok
        } else {
            return false;
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

/// `FOO = …` / `Foo::BAR = …` assignment (not a plain local).
pub fn is_const_assign(node: Node<'_>) -> bool {
    if node.kind() != "assignment" {
        return false;
    }
    node.child_by_field_name("left")
        .or_else(|| node.named_child(0))
        .is_some_and(|lhs| matches!(lhs.kind(), "constant" | "scope_resolution"))
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
