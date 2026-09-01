//! Pattern-match `=>` recovery ERRORs that MRI accepts.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

/// Multiline rightward assignment `expr =>\n  key:` — tree-sitter often splits later
/// keys into `left_assignment_list` with ERROR `:` after a preceding `match_pattern`.
pub(super) fn pattern_match_arrow_error(source: &SourceFile, node: Node<'_>) -> bool {
    node.is_error()
        && looks_like_keyword_pattern_fragment(source, node)
        && (error_in_pattern_lhs_list(source, node)
            || node
                .parent()
                .is_some_and(|p| after_match_pattern_continuation(source, p, node)))
}

fn is_ident_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn looks_like_keyword_pattern_fragment(source: &SourceFile, node: Node<'_>) -> bool {
    let t = node_bytes(source, node);
    if matches!(t, b":" | b":," | b",") {
        return true;
    }
    let Ok(s) = std::str::from_utf8(t) else {
        return false;
    };
    let Some((name, rest)) = s.trim().split_once(':') else {
        return false;
    };
    let rest = rest.trim().trim_end_matches(',').trim();
    is_ident_name(name) && (rest.is_empty() || is_ident_name(rest))
}

fn match_pattern_has_arrow(source: &SourceFile, node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.children(&mut cur)
        .any(|c| c.kind() == "=>" || node_bytes(source, c) == b"=>")
}

fn sibling_before<'a>(parent: Node<'a>, child: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = parent.walk();
    let kids: Vec<_> = parent.children(&mut cur).collect();
    let Some(idx) = kids.iter().position(|k| k.id() == child.id()) else {
        return Vec::new();
    };
    kids[..idx].to_vec()
}

fn preceding_match_pattern(source: &SourceFile, parent: Node<'_>, child: Node<'_>) -> bool {
    sibling_before(parent, child).iter().rev().any(|k| {
        k.kind() == "match_pattern" && match_pattern_has_arrow(source, *k)
    })
}

fn continuation_gap_ok(kids: &[Node<'_>]) -> bool {
    kids.iter()
        .all(|k| matches!(k.kind(), "identifier" | "comment") || k.is_error())
}

fn after_match_pattern_continuation(
    source: &SourceFile,
    parent: Node<'_>,
    child: Node<'_>,
) -> bool {
    let before = sibling_before(parent, child);
    let Some(mp) = before
        .iter()
        .rposition(|k| k.kind() == "match_pattern" && match_pattern_has_arrow(source, *k))
    else {
        return false;
    };
    continuation_gap_ok(&before[mp + 1..])
}

fn error_in_pattern_lhs_list(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(lal) = node.parent().filter(|p| p.kind() == "left_assignment_list") else {
        return false;
    };
    let Some(assign) = lal.parent().filter(|p| p.kind() == "assignment") else {
        return false;
    };
    assign
        .parent()
        .is_some_and(|body| preceding_match_pattern(source, body, assign))
}
