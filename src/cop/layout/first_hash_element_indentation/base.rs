//! Column the first hash key and closing brace are measured from.

use tree_sitter::Node;

use crate::cop::CopConfig;
use crate::cop::shared;
use crate::parse::source::SourceFile;

use super::report::paren_col;

#[derive(Clone, Copy)]
pub(super) enum Base {
    AfterParen,
    ParentKey,
    LineStart,
    LeftBrace,
}

impl Base {
    pub(super) fn description(self) -> &'static str {
        match self {
            Self::AfterParen => "the first position after the preceding left parenthesis",
            Self::ParentKey => "the parent hash key",
            Self::LineStart => "the start of the line where the left curly brace is",
            Self::LeftBrace => "the position of the opening brace",
        }
    }

    pub(super) fn brace_message(self) -> &'static str {
        match self {
            Self::AfterParen => {
                "Indent the right brace the same as the first position after the preceding left parenthesis."
            }
            Self::ParentKey => "Indent the right brace the same as the parent hash key.",
            Self::LineStart => {
                "Indent the right brace the same as the start of the line where the left brace is."
            }
            Self::LeftBrace => "Indent the right brace the same as the left brace.",
        }
    }
}

pub(super) fn braced(source: &SourceFile, hash: Node<'_>) -> bool {
    source.as_bytes().get(hash.start_byte()) == Some(&b'{')
}

pub(super) fn hash_pairs<'a>(hash: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = hash.walk();
    hash.named_children(&mut cur)
        .filter(|n| n.kind() == "pair")
        .collect()
}

pub(super) fn closing_brace(hash: Node<'_>) -> Option<Node<'_>> {
    let mut cur = hash.walk();
    hash.children(&mut cur)
        .find(|c| !c.is_named() && c.kind() == "}")
}

pub(super) fn code_before(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(start) = source.line_start(shared::node_line(source, node)) else {
        return false;
    };
    source.as_bytes()[start..node.start_byte()]
        .iter()
        .any(|b| !b.is_ascii_whitespace())
}

fn end_line(source: &SourceFile, node: Node<'_>) -> usize {
    source
        .offset_to_line_col(node.end_byte().saturating_sub(1))
        .0
}

fn next_sibling<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let parent = node.parent()?;
    let mut cur = parent.walk();
    let mut seen = false;
    for child in parent.named_children(&mut cur) {
        if child.kind() == "comment" {
            continue;
        }
        if seen {
            return Some(child);
        }
        if child.id() == node.id() {
            seen = true;
        }
    }
    None
}

fn key_value_same_line(source: &SourceFile, pair: Node<'_>) -> bool {
    let Some(key) = pair.child_by_field_name("key") else {
        return false;
    };
    let Some(value) = pair.child_by_field_name("value") else {
        return false;
    };
    shared::node_line(source, key) == shared::node_line(source, value)
}

fn parent_key_col(source: &SourceFile, hash: Node<'_>) -> Option<usize> {
    let pair = hash.parent().filter(|p| p.kind() == "pair")?;
    if !key_value_same_line(source, pair) {
        return None;
    }
    let sib = next_sibling(pair)?;
    (end_line(source, pair) < shared::node_line(source, sib))
        .then_some(shared::node_col(source, pair))
}

pub(super) fn indent_base(
    source: &SourceFile,
    hash: Node<'_>,
    config: &CopConfig,
) -> (usize, Base) {
    let style = config.get_str("EnforcedStyle", "special_inside_parentheses");
    if style == "align_braces" {
        return (shared::node_col(source, hash), Base::LeftBrace);
    }
    if let Some(col) = parent_key_col(source, hash) {
        return (col, Base::ParentKey);
    }
    if style == "special_inside_parentheses" {
        if let Some(col) = paren_col(source, hash, config) {
            return (col + 1, Base::AfterParen);
        }
    }
    (
        shared::line_indent(source, hash.start_byte()),
        Base::LineStart,
    )
}

fn pair_colon(pair: Node<'_>) -> bool {
    let mut cur = pair.walk();
    pair.children(&mut cur)
        .any(|c| !c.is_named() && c.kind() == ":")
}

fn key_len(pair: Node<'_>) -> Option<usize> {
    let key = pair.child_by_field_name("key")?;
    Some(key.end_byte() - key.start_byte())
}

pub(super) fn separator_offset(pairs: &[Node<'_>], config: &CopConfig) -> usize {
    let Some(first) = pairs.first() else {
        return 0;
    };
    let style_key = if pair_colon(*first) {
        "EnforcedColonStyle"
    } else {
        "EnforcedHashRocketStyle"
    };
    if config.get_str(style_key, "key") != "separator" {
        return 0;
    }
    let Some(first_len) = key_len(*first) else {
        return 0;
    };
    pairs
        .iter()
        .filter_map(|p| key_len(*p))
        .max()
        .unwrap_or(0)
        .saturating_sub(first_len)
}
