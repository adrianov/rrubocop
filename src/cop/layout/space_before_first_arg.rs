//! Layout/SpaceBeforeFirstArg.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceBeforeFirstArg;

fn method_node(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("method")
        .or_else(|| node.child_by_field_name("name"))
}

fn first_arg(args: Node<'_>) -> Option<Node<'_>> {
    let mut cur = args.walk();
    args.named_children(&mut cur).next()
}

fn gap_spaces(bytes: &[u8], method_end: usize, first_start: usize) -> Option<usize> {
    let between = &bytes[method_end..first_start];
    if between.iter().any(|&b| b == b'\n') {
        return None;
    }
    Some(between.iter().filter(|&&b| b == b' ' || b == b'\t').count())
}

fn line_has_nonspace_at(line: &[u8], col: usize) -> bool {
    line.get(col).is_some_and(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
}

/// RuboCop `AllowForAlignment`: extra spaces OK when arg aligns with something nearby.
fn aligned_with_adjacent(source: &SourceFile, first_start: usize) -> bool {
    let (line, col) = source.offset_to_line_col(first_start);
    if line == 0 {
        return false;
    }
    let lines: Vec<&[u8]> = source.lines().collect();
    let idx = line - 1;
    [idx.wrapping_sub(1), idx + 1]
        .into_iter()
        .any(|other| other < lines.len() && other != idx && line_has_nonspace_at(lines[other], col))
}

fn offense(source: &SourceFile, node: Node<'_>, allow_align: bool) -> Option<(usize, usize)> {
    let (method_end, first_start) = unparen_call_gap(source, node)?;
    let spaces = gap_spaces(source.as_bytes(), method_end, first_start)?;
    bad_space_gap(source, method_end, first_start, spaces, allow_align)
}

fn unparen_call_gap(source: &SourceFile, node: Node<'_>) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let method = method_node(node)?;
    let args = node.child_by_field_name("arguments")?;
    if bytes.get(args.start_byte()) == Some(&b'(') {
        return None;
    }
    let first = first_arg(args)?;
    Some((method.end_byte(), first.start_byte()))
}

fn bad_space_gap(
    source: &SourceFile,
    method_end: usize,
    first_start: usize,
    spaces: usize,
    allow_align: bool,
) -> Option<(usize, usize)> {
    if spaces == 1 {
        return None;
    }
    if spaces > 1 && allow_align && aligned_with_adjacent(source, first_start) {
        return None;
    }
    Some((method_end, first_start))
}

impl Cop for SpaceBeforeFirstArg {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeFirstArg"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "command_call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let allow = config.get_bool("AllowForAlignment", true);
        let Some((start, end)) = offense(source, node, allow) else {
            return;
        };
        report::report_fix(
            self,
            source,
            start,
            "Put one space between the method name and the first argument.".into(),
            diagnostics,
            &mut corrections,
            start,
            end,
            " ".into(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(SpaceBeforeFirstArg, "cops/layout/space_before_first_arg");
}
