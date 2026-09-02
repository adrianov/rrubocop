//! Metrics/ClassLength — class body line count vs Max.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ClassLength;

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

/// Lines strictly inside the class (between `class`/`end`), RuboCop-style.
fn body_line_count(source: &SourceFile, node: Node<'_>, count_comments: bool) -> usize {
    let start_line = node.start_position().row + 1;
    let end_line = node.end_position().row + 1;
    if end_line <= start_line + 1 {
        return 0;
    }
    source
        .lines()
        .enumerate()
        .filter(|(i, line)| {
            let ln = *i + 1;
            ln > start_line && ln < end_line && !irrelevant_line(line, count_comments)
        })
        .count()
}

fn check_class(
    cop: &ClassLength,
    source: &SourceFile,
    node: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let max = config.get_usize("Max", 100);
    let count_comments = config.get_bool("CountComments", false);
    let length = body_line_count(source, node, count_comments);
    if length <= max {
        return;
    }
    let (line, col) = source.offset_to_line_col(node.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!("Class has too many lines. [{length}/{max}]"),
    ));
}

impl Cop for ClassLength {
    fn name(&self) -> &'static str {
        "Metrics/ClassLength"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class", "singleton_class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if node.kind() == "singleton_class" {
            // RuboCop skips `class << self` nested inside another class.
            let mut p = node.parent();
            while let Some(n) = p {
                if n.kind() == "class" {
                    return;
                }
                p = n.parent();
            }
        }
        check_class(self, source, node, config, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ClassLength, "cops/metrics/class_length");
}
