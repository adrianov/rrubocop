//! Column math for Layout/FirstArgumentIndentation.

use tree_sitter::Node;

use crate::cop::layout::first_indent;
use crate::cop::shared::{self, call_method_name};
use crate::parse::source::SourceFile;

fn full_comment(source: &SourceFile, line: usize) -> bool {
    let Some(start) = source.line_start(line) else {
        return false;
    };
    let bytes = source.as_bytes();
    let mut i = start;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    i < bytes.len() && bytes[i] == b'#'
}

pub(super) fn prev_code_col(source: &SourceFile, mut line: usize) -> usize {
    loop {
        if line <= 1 {
            return 0;
        }
        line -= 1;
        if shared::line_blank(source, line) || full_comment(source, line) {
            continue;
        }
        return shared::line_indent(source, source.line_start(line).unwrap_or(0));
    }
}

fn outer_call<'a>(source: &SourceFile, call: Node<'a>) -> Option<Node<'a>> {
    let args = call.parent()?;
    if args.kind() != "argument_list" {
        return None;
    }
    let outer = args.parent()?;
    if !matches!(outer.kind(), "call" | "command" | "command_call") {
        return None;
    }
    if outer.child_by_field_name("arguments") != Some(args) {
        return None;
    }
    if call_method_name(source, outer) == Some(b"[]=") {
        return None;
    }
    (call.start_byte() > outer.start_byte()).then_some(outer)
}

fn outer_parenthesized(outer: Node<'_>) -> bool {
    outer
        .child_by_field_name("arguments")
        .is_some_and(first_indent::argument_list_opens_with_paren)
}

pub(super) fn is_special(source: &SourceFile, call: Node<'_>, style: &str) -> bool {
    if style == "consistent" {
        return false;
    }
    if style == "consistent_relative_to_receiver" {
        return true;
    }
    let Some(outer) = outer_call(source, call) else {
        return false;
    };
    match style {
        "special_for_inner_method_call" => true,
        "special_for_inner_method_call_in_parentheses" => outer_parenthesized(outer),
        _ => false,
    }
}

pub(super) fn base_text(source: &SourceFile, call: Node<'_>, arg: Node<'_>) -> String {
    let bytes = &source.as_bytes()[call.start_byte()..arg.start_byte()];
    String::from_utf8_lossy(bytes).trim().to_string()
}

pub(super) fn special_col(source: &SourceFile, call: Node<'_>, text: &str) -> usize {
    if !text.contains('\n') {
        return shared::node_col(source, call);
    }
    prev_code_col(
        source,
        shared::node_line(source, call) + text.matches('\n').count() + 1,
    )
}

pub(super) fn base_label(text: &str, special: bool) -> String {
    if special && !text.contains('\n') {
        return format!("`{text}`");
    }
    if text
        .lines()
        .next_back()
        .unwrap_or("")
        .trim_start()
        .starts_with('#')
    {
        return "the start of the previous line (not counting the comment)".to_string();
    }
    "the start of the previous line".to_string()
}
