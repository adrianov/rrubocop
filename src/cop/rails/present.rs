//! Rails/Present — `!x.blank?`, `x && !x.empty?` (also `!x.nil? &&`, `!!x &&`,
//! `x != nil &&`) and `unless x.blank?` → `x.present?` / `if x.present?`.

use std::ops::Range;

use tree_sitter::Node;

use crate::cop::shared::{Edit, child_kind, field_text, node_text, push_edits, span_diagnostic};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct Present;

/// Offense range, message, and autocorrect edits.
type Finding = (Range<usize>, String, Vec<Edit>);

impl Cop for Present {
    fn name(&self) -> &'static str {
        "Rails/Present"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["unary", "binary", "unless", "unless_modifier"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let finding = match node.kind() {
            "unary" if config.get_bool("NotBlank", true) => not_blank(source, node),
            "binary" if config.get_bool("NotNilAndNotEmpty", true) => {
                exists_not_empty(source, node)
            }
            "unless" | "unless_modifier" if config.get_bool("UnlessBlank", true) => {
                unless_blank(source, node, config)
            }
            _ => None,
        };
        let Some((range, msg, edits)) = finding else {
            return;
        };
        let mut diag = span_diagnostic(self, source, range, msg);
        diag.corrected = push_edits(&mut corrections, source, edits, self.name());
        diagnostics.push(diag);
    }
}

/// `!x.blank?` / `not x.blank?`.
fn not_blank(source: &SourceFile, node: Node<'_>) -> Option<Finding> {
    let receiver = negated_call(source, node, "blank?")?;
    Some(replace_with_present(source, node, receiver))
}

/// RuboCop `exists_and_not_empty?`: `<x exists> && !x.empty?` on the same `x`.
fn exists_not_empty(source: &SourceFile, node: Node<'_>) -> Option<Finding> {
    if !matches!(field_text(source, node, "operator"), "&&" | "and") {
        return None;
    }
    let subject = negated_call(source, node.child_by_field_name("right")?, "empty?")?;
    let exists = exists_subject(source, node.child_by_field_name("left")?);
    let text = |n: Option<Node<'_>>| n.map(|n| node_text(source, n));
    (text(exists) == text(subject)).then(|| replace_with_present(source, node, exists))
}

/// Offense on the whole `node`, autocorrected to `receiver.present?`.
fn replace_with_present(
    source: &SourceFile,
    node: Node<'_>,
    receiver: Option<Node<'_>>,
) -> Finding {
    let prefer = replacement(source, receiver);
    let msg = format!("Use `{prefer}` instead of `{}`.", node_text(source, node));
    let edit = (node.start_byte(), node.end_byte(), prefer);
    (node.byte_range(), msg, vec![edit])
}

/// What the left side asserts exists: `!x.nil?`, `!!x`, `x != nil`, else `x` itself.
fn exists_subject<'a>(source: &SourceFile, left: Node<'a>) -> Option<Node<'a>> {
    if let Some(receiver) = negated_call(source, left, "nil?") {
        return receiver;
    }
    if let Some(operand) = negated(source, left).and_then(|inner| negated(source, inner)) {
        return Some(operand);
    }
    let nil_compare = field_text(source, left, "operator") == "!="
        && left
            .child_by_field_name("right")
            .is_some_and(|r| r.kind() == "nil");
    if nil_compare {
        left.child_by_field_name("left")
    } else {
        Some(left)
    }
}

/// `unless x.blank?` (no `else` while Style/UnlessElse is enabled).
fn unless_blank(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> Option<Finding> {
    if node.child_by_field_name("alternative").is_some()
        && config.get_bool("UnlessElseEnabled", true)
    {
        return None;
    }
    let cond = node.child_by_field_name("condition")?;
    let receiver = plain_call(source, cond, "blank?")?;
    Some(if_present(
        source,
        child_kind(node, "unless")?,
        cond,
        receiver,
    ))
}

/// Offense on `unless x.blank?` (keyword through condition), autocorrected to `if x.present?`.
fn if_present(
    source: &SourceFile,
    keyword: Node<'_>,
    cond: Node<'_>,
    receiver: Option<Node<'_>>,
) -> Finding {
    let prefer = replacement(source, receiver);
    let range = keyword.start_byte()..cond.end_byte();
    let current = String::from_utf8_lossy(&source.as_bytes()[range.clone()]);
    let msg = format!("Use `if {prefer}` instead of `{current}`.");
    let edits = vec![
        (keyword.start_byte(), keyword.end_byte(), "if".to_string()),
        (cond.start_byte(), cond.end_byte(), prefer),
    ];
    (range, msg, edits)
}

/// Operand of `!x` / `not x`.
fn negated<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    (node.kind() == "unary" && matches!(field_text(source, node, "operator"), "!" | "not"))
        .then(|| node.child_by_field_name("operand"))?
}

/// Receiver (if any) of `!recv.method` / `not recv.method` (a plain send).
fn negated_call<'a>(source: &SourceFile, node: Node<'a>, method: &str) -> Option<Option<Node<'a>>> {
    plain_call(source, negated(source, node)?, method)
}

/// Receiver (if any) of an argument-less, block-less `recv.method` send (not `&.`).
fn plain_call<'a>(source: &SourceFile, node: Node<'a>, method: &str) -> Option<Option<Node<'a>>> {
    let plain = node.kind() == "call"
        && field_text(source, node, "method") == method
        && field_text(source, node, "operator") != "&."
        && node.child_by_field_name("arguments").is_none()
        && node.child_by_field_name("block").is_none();
    plain.then(|| node.child_by_field_name("receiver"))
}

fn replacement(source: &SourceFile, receiver: Option<Node<'_>>) -> String {
    receiver.map_or("present?".into(), |r| {
        format!("{}.present?", node_text(source, r))
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(Present, "cops/rails/present");

    #[test]
    fn autocorrect_matches_rubocop() {
        let src = b"!x.blank?\nfoo unless bar.blank?\n!a.nil? && !a.empty?\n";
        assert_eq!(
            crate::testutil::autocorrect_nodes(&Present, src, CopConfig::default()),
            "x.present?\nfoo if bar.present?\na.present?\n"
        );
    }
}
