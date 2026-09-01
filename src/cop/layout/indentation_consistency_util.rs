//! Column / statement-list helpers for IndentationConsistency.

use tree_sitter::Node;

use crate::cop::shared;
use crate::parse::source::SourceFile;

pub(crate) const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];
const MODS: &[&[u8]] = &[b"private", b"protected", b"public", b"module_function"];

pub(crate) fn is_stmt_list(kind: &str) -> bool {
    matches!(
        kind,
        "body_statement"
            | "begin"
            | "then"
            | "else"
            | "do"
            | "ensure"
            | "block_body"
            | "parenthesized_statements"
            | "program"
            | "begin_block"
    )
}

fn skip_stmt(kind: &str) -> bool {
    matches!(
        kind,
        "comment" | "rescue" | "ensure" | "else" | "elsif" | "when" | "in" | "pattern"
    )
}

pub(crate) fn stmt_kids<'a>(source: &SourceFile, n: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = n.walk();
    let kids: Vec<_> = n
        .named_children(&mut cur)
        .filter(|k| !skip_stmt(k.kind()))
        .collect();
    let mut seen_match = false;
    kids.into_iter()
        .filter(|k| {
            if k.kind() == "match_pattern" {
                seen_match = true;
                return true;
            }
            !(seen_match && pattern_key_recovery(source, *k))
        })
        .collect()
}

/// Tree-sitter often folds later `key:,` lines (and following stmts) into one
/// recovered `assignment` after `expr => first_key:`.
fn pattern_key_recovery(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(node.kind(), "keyword_pattern" | "hash_key_symbol")
        || source.as_bytes().get(node.end_byte()) == Some(&b':')
        || starts_with_pattern_key(source, node)
}

fn starts_with_pattern_key(source: &SourceFile, node: Node<'_>) -> bool {
    let Ok(s) = std::str::from_utf8(shared::node_bytes(source, node)) else {
        return false;
    };
    let Some((name, rest)) = s.trim_start().split_once(':') else {
        return false;
    };
    ident_name(name.trim()) && rest.starts_with(',')
}

fn ident_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub(crate) fn is_bare_access_modifier(source: &SourceFile, node: Node<'_>) -> bool {
    match node.kind() {
        "identifier" => MODS.contains(&shared::node_bytes(source, node)),
        "call" | "command" => {
            matches!(
                shared::call_method_name(source, node),
                Some(b"private" | b"protected" | b"public" | b"module_function")
            ) && node.child_by_field_name("arguments").is_none()
        }
        _ => false,
    }
}

pub(crate) fn display_col(source: &SourceFile, offset: usize) -> usize {
    let (line, col) = source.offset_to_line_col(offset);
    if line == 1 && offset >= UTF8_BOM.len() && source.as_bytes().starts_with(&UTF8_BOM) {
        col.saturating_sub(1)
    } else {
        col
    }
}

pub(crate) fn begins_its_line(source: &SourceFile, offset: usize) -> bool {
    let (line, col) = source.offset_to_line_col(offset);
    let Some(start) = source.line_start(line) else {
        return true;
    };
    let bytes = source.as_bytes();
    let end = (start + col).min(bytes.len());
    let from = if line == 1 && start == 0 && bytes.starts_with(&UTF8_BOM) {
        UTF8_BOM.len().min(end)
    } else {
        start
    };
    bytes[from..end]
        .iter()
        .all(|&b| matches!(b, b' ' | b'\t' | b'\r'))
}

pub(crate) fn end_line(source: &SourceFile, node: Node<'_>) -> usize {
    source.offset_to_line_col(node.end_byte().saturating_sub(1)).0
}

pub(crate) fn parent_column(source: &SourceFile, n: Node<'_>) -> Option<usize> {
    let parent = n.parent()?;
    if matches!(parent.kind(), "do_block" | "block") {
        if let Some(call) = parent.parent() {
            if matches!(call.kind(), "call" | "command") {
                return Some(display_col(source, call.start_byte()));
            }
        }
    }
    Some(display_col(source, parent.start_byte()))
}

pub(crate) fn base_column_for_normal(
    source: &SourceFile,
    kids: &[Node<'_>],
    parent_col: Option<usize>,
) -> Option<usize> {
    let first = *kids.first()?;
    if !is_bare_access_modifier(source, first) {
        return None;
    }
    let access_col = display_col(source, first.start_byte());
    match parent_col {
        Some(pc) => (access_col > pc).then_some(access_col),
        None => Some(access_col),
    }
}
