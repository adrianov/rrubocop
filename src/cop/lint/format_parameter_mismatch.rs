use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/FormatParameterMismatch — format/sprintf/% arg count.
pub struct FormatParameterMismatch;

fn consume_named(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'}' {
        i += 1;
    }
    i
}

fn skip_flags(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && b"-0+ #".contains(&bytes[i]) {
        i += 1;
    }
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'*' || bytes[i] == b'.') {
        i += 1;
    }
    i
}

fn find_percent(bytes: &[u8], mut i: usize) -> Option<usize> {
    while i < bytes.len() {
        if bytes[i] == b'%' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn consume_angle_named(bytes: &[u8], mut i: usize) -> usize {
    // %<name>s — skip until '>' then type char
    while i < bytes.len() && bytes[i] != b'>' {
        i += 1;
    }
    if i < bytes.len() {
        i += 1; // '>'
    }
    if i < bytes.len() {
        i += 1; // type
    }
    i
}

/// Returns (next_index, counted_field, is_named).
fn after_percent(bytes: &[u8], mut i: usize) -> (usize, bool, bool) {
    i += 1;
    if bytes.get(i) == Some(&b'%') {
        return (i + 1, false, false);
    }
    if bytes.get(i) == Some(&b'{') {
        return (consume_named(bytes, i).saturating_add(1), true, true);
    }
    if bytes.get(i) == Some(&b'<') {
        return (consume_angle_named(bytes, i + 1), true, true);
    }
    i = skip_flags(bytes, i);
    if i >= bytes.len() {
        (i, false, false)
    } else {
        (i + 1, true, false)
    }
}

fn scan_percent(bytes: &[u8], i: usize) -> Option<(usize, bool, bool)> {
    let i = find_percent(bytes, i)?;
    Some(after_percent(bytes, i))
}

/// Named formats (`%{x}` / `%<x>s`) expect one hash/kwargs argument (RuboCop).
fn field_count(fmt: &str) -> Option<(usize, bool)> {
    let bytes = fmt.as_bytes();
    let mut i = 0;
    let mut count = 0;
    let mut named = false;
    while let Some((next, counted, is_named)) = scan_percent(bytes, i) {
        if counted {
            count += 1;
        }
        named |= is_named;
        i = next;
    }
    Some((count, named))
}

fn string_content(source: &SourceFile, node: Node<'_>) -> Option<String> {
    if node.kind() != "string" {
        return None;
    }
    let mut cur = node.walk();
    let mut out = String::new();
    for c in node.named_children(&mut cur) {
        if c.kind() == "string_content" {
            out.push_str(&node_text(source, c));
        } else if c.kind() == "interpolation" {
            return None;
        }
    }
    Some(out)
}

fn array_len(node: Node<'_>) -> usize {
    let mut cur = node.walk();
    node.named_children(&mut cur).count()
}

fn count_format_args(args: &[Node<'_>]) -> usize {
    if args.is_empty() {
        return 0;
    }
    // Trailing keywords / hash → one arg (RuboCop / MRI format).
    let all_pairs = args.iter().all(|a| a.kind() == "pair" || a.kind() == "hash");
    if all_pairs {
        return 1;
    }
    let has_kw = args.iter().any(|a| a.kind() == "pair");
    args.iter().filter(|a| a.kind() != "pair").count() + usize::from(has_kw)
}

fn report_mismatch(
    cop: &FormatParameterMismatch,
    source: &SourceFile,
    node: Node<'_>,
    method: &str,
    actual: usize,
    expected: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if actual == expected {
        return;
    }
    let (line, col) = source.offset_to_line_col(node.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!(
            "Number of arguments ({actual}) to `{method}` doesn't match the number of fields ({expected})."
        ),
    ));
}

fn check_percent(
    cop: &FormatParameterMismatch,
    source: &SourceFile,
    node: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(op) = node.child_by_field_name("operator") else {
        return;
    };
    if node_bytes(source, op) != b"%" {
        return;
    }
    let Some(left) = node.child_by_field_name("left") else {
        return;
    };
    let Some(right) = node.child_by_field_name("right") else {
        return;
    };
    let Some(fmt) = string_content(source, left) else {
        return;
    };
    let Some((expected, named)) = field_count(&fmt) else {
        return;
    };
    let actual = if right.kind() == "array" {
        array_len(right)
    } else if named {
        1
    } else {
        1
    };
    let expected = if named { 1 } else { expected };
    report_mismatch(cop, source, node, "String#%", actual, expected, diagnostics);
}

fn check_format(
    cop: &FormatParameterMismatch,
    source: &SourceFile,
    node: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(meth) = call_method_name(source, node) else {
        return;
    };
    if meth != b"format" && meth != b"sprintf" {
        return;
    }
    let _ = call_receiver(node);
    let args = argument_nodes(node);
    if args.is_empty() {
        return;
    }
    let Some(fmt) = string_content(source, args[0]) else {
        return;
    };
    let Some((expected, named)) = field_count(&fmt) else {
        return;
    };
    let rest = &args[1..];
    let actual = if named {
        if rest.is_empty() {
            0
        } else {
            1
        }
    } else {
        // Keyword args / trailing hash count as one argument (Ruby format).
        count_format_args(rest)
    };
    let expected = if named { 1 } else { expected };
    let method = String::from_utf8_lossy(meth);
    report_mismatch(cop, source, node, &method, actual, expected, diagnostics);
}

impl Cop for FormatParameterMismatch {
    fn name(&self) -> &'static str {
        "Lint/FormatParameterMismatch"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "binary"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.kind() == "binary" {
            check_percent(self, source, node, diagnostics);
        } else {
            check_format(self, source, node, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(FormatParameterMismatch, "cops/lint/format_parameter_mismatch");
}
