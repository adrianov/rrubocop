//! Metrics/ModuleLength — module body line count vs Max.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ModuleLength;

fn irrelevant_line(line: &[u8], count_comments: bool) -> bool {
    let trimmed = trim_ascii(line);
    trimmed.is_empty() || (!count_comments && trimmed.starts_with(b"#"))
}

fn trim_ascii(line: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < line.len() && matches!(line[start], b' ' | b'\t') {
        start += 1;
    }
    let mut end = line.len();
    while end > start && matches!(line[end - 1], b' ' | b'\t' | b'\r') {
        end -= 1;
    }
    &line[start..end]
}

fn nested_skip_ranges(node: Node<'_>) -> Vec<(usize, usize)> {
    let Some(body) = node.child_by_field_name("body") else {
        return Vec::new();
    };
    let mut cur = body.walk();
    body.named_children(&mut cur)
        .filter(|c| matches!(c.kind(), "class" | "module" | "singleton_class"))
        .map(|c| (c.start_position().row + 1, c.end_position().row + 1))
        .collect()
}

fn line_counted(
    ln: usize,
    start_line: usize,
    end_line: usize,
    line: &[u8],
    count_comments: bool,
    skip: &[(usize, usize)],
) -> bool {
    ln > start_line
        && ln < end_line
        && !irrelevant_line(line, count_comments)
        && !skip.iter().any(|&(s, e)| ln >= s && ln <= e)
}

fn body_line_count(source: &SourceFile, node: Node<'_>, count_comments: bool) -> usize {
    let start_line = node.start_position().row + 1;
    let end_line = node.end_position().row + 1;
    if end_line <= start_line + 1 {
        return 0;
    }
    let skip = nested_skip_ranges(node);
    source
        .lines()
        .enumerate()
        .filter(|(i, line)| line_counted(*i + 1, start_line, end_line, line, count_comments, &skip))
        .count()
}

fn is_namespace_module(node: Node<'_>) -> bool {
    let Some(body) = node.child_by_field_name("body") else {
        return false;
    };
    let mut cur = body.walk();
    let kids: Vec<_> = body.named_children(&mut cur).collect();
    kids.len() == 1 && matches!(kids[0].kind(), "class" | "module" | "singleton_class")
}

impl Cop for ModuleLength {
    fn name(&self) -> &'static str {
        "Metrics/ModuleLength"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["module"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if is_namespace_module(node) {
            return;
        }
        let max = config.get_usize("Max", 100);
        let length = body_line_count(source, node, config.get_bool("CountComments", false));
        if length <= max {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Module has too many lines. [{length}/{max}]"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ModuleLength, "cops/metrics/module_length");
}
