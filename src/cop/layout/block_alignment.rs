//! Layout/BlockAlignment.

use tree_sitter::Node;

use crate::cop::layout::indentation_consistency_util;
use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BlockAlignment;

fn block_opener<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let kw = if node.kind() == "do_block" { "do" } else { "{" };
    let mut cur = node.walk();
    node.children(&mut cur)
        .find(|c| !c.is_named() && c.kind() == kw)
}

fn brace_block_in_stabby_lambda(source: &SourceFile, opener: Node<'_>) -> bool {
    if opener.kind() != "{" {
        return false;
    }
    let Some(line_start) = source.line_start(source.offset_to_line_col(opener.start_byte()).0) else {
        return false;
    };
    let line_end = source
        .lines()
        .nth(source.offset_to_line_col(opener.start_byte()).0.saturating_sub(1))
        .map(|l| line_start + l.len())
        .unwrap_or(opener.start_byte());
    let bytes = source.as_bytes();
    let slice = &bytes[line_start..line_end.min(bytes.len())];
    slice.windows(2).any(|w| w == b"->")
}

fn block_call_node<'a>(block: Node<'a>) -> Option<Node<'a>> {
    let parent = block.parent()?;
    if !matches!(parent.kind(), "call" | "command" | "command_call") {
        return None;
    }
    parent
        .child_by_field_name("block")
        .filter(|b| b.id() == block.id())
        .map(|_| parent)
}

fn first_non_ws_offset(source: &SourceFile, line: usize) -> Option<usize> {
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

fn inside_parentheses(source: &SourceFile, call: Node<'_>, pos: usize) -> bool {
    let bytes = source.as_bytes();
    let start = call.start_byte();
    if pos <= start {
        return false;
    }
    let mut opens = 0i32;
    let mut closes = 0i32;
    for &b in &bytes[start..pos.min(bytes.len())] {
        match b {
            b'(' | b'[' => opens += 1,
            b')' | b']' => closes += 1,
            _ => {}
        }
    }
    opens > closes
}

fn do_line_begins_inside_argument(source: &SourceFile, call: Node<'_>, opener_off: usize) -> bool {
    let (line, _) = source.offset_to_line_col(opener_off);
    let Some(first_char_off) = first_non_ws_offset(source, line) else {
        return false;
    };
    if !inside_parentheses(source, call, first_char_off) {
        return false;
    }
    let Some(args) = call.child_by_field_name("arguments") else {
        return false;
    };
    let mut cur = args.walk();
    args.named_children(&mut cur).any(|arg| {
        arg.start_byte() <= first_char_off && first_char_off < arg.end_byte()
    })
}

fn method_selector_offset(call: Node<'_>) -> usize {
    call.child_by_field_name("method")
        .map(|m| m.start_byte())
        .unwrap_or_else(|| call.start_byte())
}

/// When `do`/`{` sits on a multiline argument continuation line, anchor on the call.
fn do_line_anchor_offset(source: &SourceFile, block: Node<'_>, opener: Node<'_>) -> usize {
    let Some(call) = block_call_node(block) else {
        return opener.start_byte();
    };
    if do_line_begins_inside_argument(source, call, opener.start_byte()) {
        method_selector_offset(call)
    } else {
        opener.start_byte()
    }
}

fn call_chain_root(mut call: Node<'_>) -> Node<'_> {
    while let Some(recv) = call.child_by_field_name("receiver") {
        if !matches!(recv.kind(), "call" | "command" | "command_call") {
            break;
        }
        call = recv;
    }
    call
}

/// RuboCop block nodes span the full send chain; tree-sitter `do_block` starts at `do`.
fn block_expression_start<'a>(block: Node<'a>) -> Node<'a> {
    block_call_node(block)
        .map(call_chain_root)
        .unwrap_or(block)
}

fn block_line_indent(source: &SourceFile, node: Node<'_>) -> usize {
    let start = block_expression_start(node);
    shared::line_indent(source, start.start_byte())
}

fn expression_start_col(source: &SourceFile, block: Node<'_>) -> usize {
    expression_start_from_ancestors(source, block_expression_start(block))
}

fn do_line_begin_col(source: &SourceFile, opener: Node<'_>) -> usize {
    let (line, _) = source.offset_to_line_col(opener.start_byte());
    first_non_ws_offset(source, line)
        .map(|off| source.offset_to_line_col(off).1)
        .unwrap_or_else(|| shared::line_indent(source, opener.start_byte()))
}

fn same_line(source: &SourceFile, a: Node<'_>, b: Node<'_>) -> bool {
    shared::node_line(source, a) == shared::node_line(source, b)
}

fn disqualified_parent(source: &SourceFile, parent: Node<'_>, current: Node<'_>) -> bool {
    !same_line(source, parent, current) && parent.kind() != "assignment"
}

fn is_assignment_parent(node: Node<'_>, source: &SourceFile) -> bool {
    match node.kind() {
        "assignment" | "operator_assignment" | "conditional_assignment" => true,
        "call" | "command" | "command_call" => shared::call_method_name(source, node)
            .is_some_and(|name| matches!(name, b"=" | b"+=" | b"-=" | b"*=" | b"/=")),
        _ => false,
    }
}

fn block_end_align_target(source: &SourceFile, parent: Node<'_>, current: Node<'_>) -> bool {
    if is_assignment_parent(parent, source) {
        return true;
    }
    match parent.kind() {
        "and" | "or" | "unary" | "splat_argument" => return true,
        "method" | "singleton_method" => return true,
        _ => {}
    }
    if !matches!(parent.kind(), "call" | "command" | "command_call") {
        return false;
    }
    if shared::call_method_name(source, parent) == Some(b"<<") {
        return true;
    }
    parent
        .child_by_field_name("receiver")
        .is_some_and(|recv| recv.id() == current.id())
        && shared::call_method_name(source, parent) != Some(b"[]")
}

fn expression_start_from_ancestors(source: &SourceFile, block: Node<'_>) -> usize {
    let mut current = block;
    let mut start_offset = block.start_byte();
    let initial = block;

    while let Some(parent) = current.parent() {
        if current.id() != initial.id()
            && parent
                .child_by_field_name("block")
                .is_some_and(|b| b.id() == current.id())
        {
            break;
        }
        if disqualified_parent(source, parent, current)
            || !block_end_align_target(source, parent, current)
        {
            break;
        }
        start_offset = parent.start_byte();
        current = parent;
    }

    source.offset_to_line_col(start_offset).1
}

fn closer_follows_rescue_modifier(source: &SourceFile, closer: Node<'_>) -> bool {
    let bytes = source.as_bytes();
    let mut pos = closer.end_byte();
    while pos < bytes.len() && matches!(bytes[pos], b' ' | b'\t' | b'\r') {
        pos += 1;
    }
    pos + 6 <= bytes.len()
        && &bytes[pos..pos + 6] == b"rescue"
        && (pos == 0
            || !bytes[pos - 1].is_ascii_alphanumeric() && bytes[pos - 1] != b'_')
        && (pos + 6 >= bytes.len()
            || (!bytes[pos + 6].is_ascii_alphanumeric() && bytes[pos + 6] != b'_'))
}

fn call_expression_col_on_opener_line(source: &SourceFile, opener: Node<'_>) -> usize {
    let bytes = source.as_bytes();
    let mut pos = opener.start_byte();
    let mut line_start = pos;
    while line_start > 0 && bytes[line_start - 1] != b'\n' {
        line_start -= 1;
    }
    while pos > line_start && bytes[pos - 1] == b' ' {
        pos -= 1;
    }
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    while pos > line_start {
        let ch = bytes[pos - 1];
        match ch {
            b')' | b']' => {
                paren_depth += 1;
                pos -= 1;
            }
            b'}' => {
                brace_depth += 1;
                pos -= 1;
            }
            b'(' | b'[' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                    pos -= 1;
                } else {
                    break;
                }
            }
            b'{' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                    pos -= 1;
                } else {
                    break;
                }
            }
            _ if paren_depth > 0 || brace_depth > 0 => pos -= 1,
            _ if ch.is_ascii_alphanumeric()
                || matches!(ch, b'_' | b'.' | b'?' | b'!' | b'@' | b'$' | b'%') =>
            {
                pos -= 1;
            }
            b':' if pos >= 2 + line_start && bytes[pos - 2] == b':' => pos -= 2,
            _ => break,
        }
    }
    let call_pos = pos;
    if call_pos > line_start {
        pos = skip_assignment_lhs(source, line_start, call_pos);
    }
    pos - line_start
}

fn skip_assignment_lhs(source: &SourceFile, line_start: usize, pos: usize) -> usize {
    let bytes = source.as_bytes();
    let mut p = pos;
    while p > line_start && bytes[p - 1] == b' ' {
        p -= 1;
    }
    if p <= line_start || bytes[p - 1] != b'=' {
        return pos;
    }
    let eq_pos = p - 1;
    if eq_pos > line_start && matches!(bytes[eq_pos - 1], b'=' | b'!' | b'<' | b'>') {
        return pos;
    }
    if eq_pos > line_start {
        let prev = bytes[eq_pos - 1];
        if matches!(prev, b'+' | b'-' | b'*' | b'/' | b'%' | b'^' | b'|' | b'&' | b'<') {
            return pos;
        }
    }
    let mut lhs_end = eq_pos;
    while lhs_end > line_start && bytes[lhs_end - 1] == b' ' {
        lhs_end -= 1;
    }
    let mut lhs_pos = lhs_end;
    while lhs_pos > line_start {
        let ch = bytes[lhs_pos - 1];
        if ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'@' | b'$' | b'.' | b'[') {
            lhs_pos -= 1;
        } else if ch == b':' {
            lhs_pos -= 1;
            if lhs_pos > line_start && bytes[lhs_pos - 1] == b':' {
                lhs_pos -= 1;
            }
        } else if ch == b',' {
            lhs_pos -= 1;
            while lhs_pos > line_start && bytes[lhs_pos - 1] == b' ' {
                lhs_pos -= 1;
            }
        } else {
            break;
        }
    }
    if lhs_pos < lhs_end {
        lhs_pos
    } else {
        pos
    }
}

fn end_aligned(
    style: &str,
    end_col: usize,
    expression_col: usize,
    do_line_col: usize,
    do_line_begin_col: usize,
    call_expr_col: usize,
) -> bool {
    match style {
        "start_of_block" => end_col == do_line_col,
        "start_of_line" => end_col == expression_col,
        _ => {
            end_col == expression_col
                || end_col == do_line_col
                || end_col == do_line_begin_col
                || end_col == call_expr_col
        }
    }
}

fn autocorrect_col(
    style: &str,
    expression_col: usize,
    do_line_col: usize,
    opener_col: usize,
) -> usize {
    match style {
        "start_of_block" => do_line_col,
        "start_of_line" => expression_col,
        _ => expression_col.min(do_line_col).min(opener_col),
    }
}

impl Cop for BlockAlignment {
    fn name(&self) -> &'static str {
        "Layout/BlockAlignment"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["do_block", "block"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(end_kw) = shared::end_keyword(node)
            .or_else(|| shared::last_child_kind(node, "}"))
        else {
            return;
        };
        if !indentation_consistency_util::begins_its_line(source, end_kw.start_byte()) {
            return;
        }
        if shared::node_line(source, node) == shared::node_line(source, end_kw) {
            return;
        }
        let Some(opener) = block_opener(node) else {
            return;
        };
        if brace_block_in_stabby_lambda(source, opener) {
            return;
        }

        let style = config.get_str("EnforcedStyleAlignWith", "either");
        let anchor_off = do_line_anchor_offset(source, node, opener);
        let do_line_col = shared::line_indent(source, anchor_off);
        let do_line_begin_col = do_line_begin_col(source, opener);
        let opener_col = shared::node_col(source, opener);
        let call_expr_col = call_expression_col_on_opener_line(source, opener);
        let mut expression_col = if style == "start_of_line" {
            expression_start_for_line(source, node)
        } else {
            expression_start_col(source, node)
        };
        if closer_follows_rescue_modifier(source, end_kw) {
            expression_col = expression_col.min(block_line_indent(source, node));
        }

        if end_aligned(
            style,
            shared::node_col(source, end_kw),
            expression_col,
            do_line_col,
            do_line_begin_col,
            call_expr_col,
        ) {
            return;
        }

        let open_word = if opener.kind() == "do" { "`do`" } else { "`{`" };
        let expected = autocorrect_col(style, expression_col, do_line_col, opener_col);
        report::fix_indent(
            self,
            source,
            end_kw.start_byte(),
            format!("`end` is not aligned with {open_word} beginning at column {opener_col}."),
            diagnostics,
            &mut corrections,
            shared::line_indent(source, end_kw.start_byte()),
            expected,
        );
    }
}

fn expression_start_for_line(source: &SourceFile, block: Node<'_>) -> usize {
    let mut start = block_expression_start(block);
    let mut col = expression_start_col(source, block);
    let mut current = start;
    while let Some(parent) = current.parent() {
        if !same_line(source, start, parent) {
            break;
        }
        col = shared::node_col(source, parent).min(shared::line_indent(source, parent.start_byte()));
        start = parent;
        current = parent;
    }
    col
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(BlockAlignment, "cops/layout/block_alignment");
}
