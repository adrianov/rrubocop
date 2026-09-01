//! Layout/MultilineOperationIndentation.

use tree_sitter::Node;

use crate::cop::layout::indentation_consistency_util as util;
use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MultilineOperationIndentation;

fn node_end_line(source: &SourceFile, node: Node<'_>) -> usize {
    source
        .offset_to_line_col(node.end_byte().saturating_sub(1))
        .0
}

fn within_node(inner: Node<'_>, outer: Node<'_>) -> bool {
    inner.start_byte() >= outer.start_byte() && inner.end_byte() <= outer.end_byte()
}

/// RuboCop skips ops inside `(...)` groups and parenthesized argument lists.
fn not_for_this_cop(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(n) = p {
        if n.kind() == "parenthesized_statements" {
            return true;
        }
        if n.kind() == "argument_list" {
            let mut cur = n.walk();
            if n.children(&mut cur).any(|c| !c.is_named() && c.kind() == "(") {
                return true;
            }
        }
        p = n.parent();
    }
    false
}

fn line_indent_bytes(line: &[u8]) -> usize {
    line.iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count()
}

fn last_significant_index(line: &[u8]) -> Option<usize> {
    line.iter()
        .rposition(|&b| b != b' ' && b != b'\t' && b != b'\r' && b != b'\n')
}

fn is_assignment_operator(bytes: &[u8], idx: usize) -> bool {
    if bytes.get(idx) != Some(&b'=') {
        return false;
    }
    if bytes.get(idx + 1) == Some(&b'=') {
        return false;
    }
    if bytes.get(idx + 1) == Some(&b'>') {
        return false;
    }
    !matches!(
        idx.checked_sub(1).and_then(|i| bytes.get(i)),
        Some(b'=' | b'!' | b'<' | b'>')
    )
}

fn has_assignment_before_col(line: &[u8], col: usize) -> bool {
    let end = col.min(line.len());
    (0..end)
        .rev()
        .find(|&idx| line[idx] == b'=')
        .is_some_and(|idx| is_assignment_operator(line, idx))
}

fn line_ends_with_assignment(line: &[u8]) -> bool {
    let mut idx = match last_significant_index(line) {
        Some(idx) => idx,
        None => return false,
    };
    if line[idx] == b'\\' {
        idx = match last_significant_index(&line[..idx]) {
            Some(idx) => idx,
            None => return false,
        };
    }
    is_assignment_operator(line, idx)
}

fn line_ends_with_logical(line: &[u8]) -> bool {
    let Some(idx) = last_significant_index(line) else {
        return false;
    };
    let trimmed = &line[..=idx];
    trimmed.ends_with(b"&&")
        || trimmed.ends_with(b"||")
        || trimmed.ends_with(b" and")
        || trimmed.ends_with(b" or")
}

#[derive(Clone, Copy)]
struct KeywordContext {
    special_indent: bool,
}

fn modifier_keyword(before: &[u8]) -> Option<KeywordContext> {
    if before.windows(8).any(|w| w == b" unless ")
        || before.windows(8).any(|w| w == b" unless(")
    {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    if before.windows(7).any(|w| w == b" while ")
        || before.windows(7).any(|w| w == b" while(")
    {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    if before.windows(7).any(|w| w == b" until ")
        || before.windows(7).any(|w| w == b" until(")
    {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    if before.windows(4).any(|w| w == b" if ") || before.windows(4).any(|w| w == b" if(") {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    None
}

fn keyword_on_line(line: &[u8], expr_col: usize) -> Option<KeywordContext> {
    let start = line_indent_bytes(line);
    let end = expr_col.min(line.len());
    let before = &line[start..end];
    if before.starts_with(b"elsif ") {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    if before.starts_with(b"if ") || before.starts_with(b"if(") {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    if before.starts_with(b"unless ") || before.starts_with(b"unless(") {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    if before.starts_with(b"while ") || before.starts_with(b"while(") {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    if before.starts_with(b"until ") || before.starts_with(b"until(") {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    if before.starts_with(b"for ") {
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    if before.starts_with(b"return ") {
        if modifier_keyword(before).is_some() {
            return Some(KeywordContext {
                special_indent: false,
            });
        }
        return Some(KeywordContext {
            special_indent: true,
        });
    }
    modifier_keyword(before)
}

fn keyword_context(source: &SourceFile, node: Node<'_>, left_line: usize, left_col: usize) -> Option<KeywordContext> {
    if let Some(ctx) = keyword_from_ancestors(node) {
        return Some(ctx);
    }

    let line = source.lines().nth(left_line.saturating_sub(1)).unwrap_or(b"");
    if let Some(ctx) = keyword_on_line(line, left_col) {
        return Some(ctx);
    }

    if left_line <= 1 {
        return None;
    }
    let prev = source.lines().nth(left_line - 2).unwrap_or(b"");
    if last_significant_index(prev).is_some_and(|idx| prev[idx] == b'\\') {
        return keyword_on_line(prev, prev.len());
    }
    let line_indent = line_indent_bytes(line);
    let prev_indent = line_indent_bytes(prev);
    if prev_indent < line_indent && line_ends_with_logical(prev) {
        return keyword_on_line(prev, prev.len());
    }
    None
}

fn keyword_from_ancestors(node: Node<'_>) -> Option<KeywordContext> {
    let mut p = node.parent();
    while let Some(anc) = p {
        match anc.kind() {
            "if" | "unless" | "while" | "until" | "elsif" => {
                if let Some(cond) = anc.child_by_field_name("condition") {
                    if within_node(node, cond) {
                        return Some(KeywordContext {
                            special_indent: true,
                        });
                    }
                }
            }
            "if_modifier" | "unless_modifier" | "while_modifier" | "until_modifier" => {
                if let Some(cond) = anc.child_by_field_name("condition") {
                    if within_node(node, cond) {
                        return Some(KeywordContext {
                            special_indent: false,
                        });
                    }
                }
            }
            _ => {}
        }
        p = anc.parent();
    }
    None
}

fn is_unaligned_rhs_type(kind: &str) -> bool {
    matches!(
        kind,
        "if" | "unless" | "while" | "until" | "for" | "return" | "array" | "begin"
    )
}

fn within_condition(node: Node<'_>, anc: Node<'_>) -> bool {
    anc.child_by_field_name("condition")
        .is_some_and(|cond| within_node(node, cond))
}

#[derive(Clone, Copy)]
struct AssignmentContext {
    rhs_begins_line: bool,
}

fn assignment_context(source: &SourceFile, left_line: usize, left_col: usize) -> Option<AssignmentContext> {
    let line = source.lines().nth(left_line.saturating_sub(1)).unwrap_or(b"");
    if has_assignment_before_col(line, left_col) {
        return Some(AssignmentContext {
            rhs_begins_line: false,
        });
    }
    if left_line > 1 {
        let prev = source.lines().nth(left_line - 2).unwrap_or(b"");
        if line_ends_with_assignment(prev)
            && left_col == line_indent_bytes(line)
        {
            return Some(AssignmentContext {
                rhs_begins_line: true,
            });
        }
    }
    None
}

fn assignment_from_ancestors(source: &SourceFile, node: Node<'_>) -> Option<AssignmentContext> {
    let mut p = node.parent();
    let mut block_disqualifies = false;
    while let Some(anc) = p {
        if matches!(anc.kind(), "do_block" | "block" | "block_body" | "lambda") {
            block_disqualifies = true;
        }
        if is_unaligned_rhs_type(anc.kind()) && !within_condition(node, anc) {
            block_disqualifies = true;
        }
        if matches!(anc.kind(), "assignment" | "operator_assignment") {
            if block_disqualifies {
                return None;
            }
            let rhs = anc.child_by_field_name("right")?;
            if within_node(node, rhs) {
                return Some(AssignmentContext {
                    rhs_begins_line: util::begins_its_line(source, rhs.start_byte()),
                });
            }
            return None;
        }
        if matches!(
            anc.kind(),
            "program" | "method" | "singleton_method" | "class" | "module"
        ) {
            break;
        }
        p = anc.parent();
    }
    None
}

impl Cop for MultilineOperationIndentation {
    fn name(&self) -> &'static str {
        "Layout/MultilineOperationIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["binary"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if not_for_this_cop(node) {
            return;
        }
        let Some((actual, expected, align_only)) = indent_mismatch(source, node, config) else {
            return;
        };
        let right = node.child_by_field_name("right").unwrap();
        let width = config.get_usize("IndentationWidth", 2);
        let left = node.child_by_field_name("left").unwrap();
        let left_indent = shared::line_indent(source, left.start_byte());
        let msg = if align_only {
            "Align the operands of a multi-line operation.".into()
        } else {
            let used = actual.saturating_sub(left_indent);
            let want = expected.saturating_sub(left_indent);
            format!(
                "Use {want} (not {used}) spaces for indenting a multi-line operation."
            )
        };
        let _ = width;
        report::fix_indent(
            self,
            source,
            right.start_byte(),
            msg,
            diagnostics,
            &mut corrections,
            actual,
            expected,
        );
    }
}

fn multiline_rhs_candidate(source: &SourceFile, left: Node<'_>, right: Node<'_>) -> bool {
    node_end_line(source, left) != shared::node_line(source, right)
        && util::begins_its_line(source, right.start_byte())
        && shared::node_col(source, right) == shared::line_indent(source, right.start_byte())
}

fn operation_expected_col(
    source: &SourceFile,
    node: Node<'_>,
    left: Node<'_>,
    width: usize,
    style: &str,
) -> (usize, bool) {
    let left_line = shared::node_line(source, left);
    let left_col = shared::node_col(source, left);
    let keyword_ctx = keyword_context(source, node, left_line, left_col);
    let assignment_ctx = assignment_from_ancestors(source, node)
        .or_else(|| assignment_context(source, left_line, left_col));
    let should_align = assignment_ctx.is_some_and(|c| c.rhs_begins_line)
        || (style == "aligned" && (keyword_ctx.is_some() || assignment_ctx.is_some()));
    if should_align {
        (left_col, true)
    } else {
        (indented_anchor_col(source, left, width, keyword_ctx.as_ref()), false)
    }
}

fn indent_mismatch(
    source: &SourceFile,
    node: Node<'_>,
    config: &CopConfig,
) -> Option<(usize, usize, bool)> {
    let width = config.get_usize("IndentationWidth", 2);
    let style = config.get_str("EnforcedStyle", "aligned");
    let left = node.child_by_field_name("left")?;
    let right = node.child_by_field_name("right")?;
    if !multiline_rhs_candidate(source, left, right) {
        return None;
    }
    let actual = shared::line_indent(source, right.start_byte());
    let (expected, align_only) = operation_expected_col(source, node, left, width, style);
    (actual != expected).then_some((actual, expected, align_only))
}

fn keyword_extra(width: usize, keyword_ctx: Option<&KeywordContext>) -> usize {
    keyword_ctx
        .filter(|c| c.special_indent)
        .map(|_| width)
        .unwrap_or(0)
}

fn indented_anchor_col(source: &SourceFile, left: Node<'_>, width: usize, keyword_ctx: Option<&KeywordContext>) -> usize {
    shared::line_indent(source, left.start_byte()) + width + keyword_extra(width, keyword_ctx)
}

/// Expected column for the method-name part of a multiline dotted call (`aligned` style).
pub(crate) fn aligned_method_call_col(
    source: &SourceFile,
    call: Node<'_>,
    receiver: Node<'_>,
    width: usize,
) -> usize {
    let left_line = shared::node_line(source, receiver);
    let left_col = shared::node_col(source, receiver);
    let keyword_ctx = keyword_context(source, call, left_line, left_col);
    let assignment_ctx = assignment_from_ancestors(source, call)
        .or_else(|| assignment_context(source, left_line, left_col));
    let should_align = assignment_ctx.is_some_and(|c| c.rhs_begins_line)
        || keyword_ctx.is_some()
        || assignment_ctx.is_some();
    if should_align {
        left_col
    } else {
        indented_anchor_col(source, receiver, width, keyword_ctx.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::run_cop_full_with_config;
    use std::collections::HashMap;

    crate::cop_fixture_tests!(
        MultilineOperationIndentation,
        "cops/layout/multiline_operation_indentation"
    );

    fn indented_config() -> CopConfig {
        CopConfig {
            options: HashMap::from([(
                "EnforcedStyle".into(),
                serde_yml::Value::String("indented".into()),
            )]),
            ..CopConfig::default()
        }
    }

    #[test]
    fn indented_if_or_chain_no_offense() {
        let diags = run_cop_full_with_config(
            &MultilineOperationIndentation,
            b"        if currency_id.blank? || !Currency.exists?(id: currency_id) ||\n            StringIdVersion.exists?(item_type: 'Currency')\n          stats[:skipped] += 1\n        end\n",
            indented_config(),
        );
        assert!(
            diags.is_empty(),
            "indented if-condition || continuation: {:?}",
            diags
        );
    }

    #[test]
    fn indented_return_unless_or_chain_no_offense() {
        let diags = run_cop_full_with_config(
            &MultilineOperationIndentation,
            b"      return unless ::Merchants::Firekassa::MERCHANT_TIDS.present? &&\n        merchant_tids.all? { |tid| withdraw_tids.include?(tid) }\n",
            indented_config(),
        );
        assert!(
            diags.is_empty(),
            "return unless || continuation: {:?}",
            diags
        );
    }

    #[test]
    fn indented_assignment_or_continuation_no_offense() {
        let diags = run_cop_full_with_config(
            &MultilineOperationIndentation,
            b"    wallet = Wallet.fee.find_by(blockchain_key: blockchain.real_key, tag: tag) ||\n      Wallet.deposit.find_by(blockchain_key: blockchain.real_key, tag: tag)\n",
            indented_config(),
        );
        assert!(
            diags.is_empty(),
            "indented assignment || continuation: {:?}",
            diags
        );
    }

    #[test]
    fn indented_assignment_plus_chain_no_offense() {
        let diags = run_cop_full_with_config(
            &MultilineOperationIndentation,
            b"    line = headers['TS-API-TIMESTAMP'].to_s +\n      headers['TS-API-API-KEY'].to_s +\n      payload\n",
            indented_config(),
        );
        assert!(
            diags.is_empty(),
            "indented assignment + chain: {:?}",
            diags
        );
    }
}
