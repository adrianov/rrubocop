//! GraphQL/ResolverMethodLength — resolver methods must stay short.

use tree_sitter::Node;

use super::helpers::{
    class_body_stmts, enclosing_class, plain_field_definition, DEPT_INCLUDE,
};
use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ResolverMethodLength;

impl Cop for ResolverMethodLength {
    fn name(&self) -> &'static str {
        "GraphQL/ResolverMethodLength"
    }

    fn default_include(&self) -> &'static [&'static str] {
        DEPT_INCLUDE
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some((length, max)) = over_limit(source, node, config) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("ResolverMethod has too many lines. [{length}/{max}]"),
        ));
    }
}

fn over_limit(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> Option<(usize, usize)> {
    let name = node_text(source, node.child_by_field_name("name")?);
    if excluded(config, &name) {
        return None;
    }
    let class = enclosing_class(node)?;
    if !resolves_plain_field(source, class, &name) {
        return None;
    }
    let max = config.get_usize("Max", 10);
    let line_count = method_line_count(node);
    if line_count <= max {
        return None;
    }
    let count_comments = config.get_bool("CountComments", false);
    let length = method_body_line_count(source, node, count_comments);
    (length > max).then_some((length, max))
}

fn excluded(config: &CopConfig, name: &str) -> bool {
    super::helpers::config_string_list(config, "ExcludedMethods")
        .iter()
        .any(|e| e == name)
}

fn resolves_plain_field(source: &SourceFile, class: Node<'_>, name: &str) -> bool {
    class_body_stmts(class).into_iter().any(|n| {
        plain_field_definition(source, n)
            && super::helpers::field_name(source, n).as_deref() == Some(name)
    })
}

fn method_line_count(node: Node<'_>) -> usize {
    node.end_position().row.saturating_sub(node.start_position().row) + 1
}

fn method_body_line_count(source: &SourceFile, node: Node<'_>, count_comments: bool) -> usize {
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

fn irrelevant_line(line: &[u8], count_comments: bool) -> bool {
    let code = strip_line_comment(line);
    let trimmed = trim_ascii_end(code);
    trimmed.is_empty() || (!count_comments && trimmed.starts_with(b"#"))
}

fn strip_line_comment(line: &[u8]) -> &[u8] {
    match crate::parse::comment_hash::first_comment_hash(line) {
        Some(i) => &line[..i],
        None => line,
    }
}

fn trim_ascii_end(code: &[u8]) -> &[u8] {
    let mut end = code.len();
    while end > 0 && matches!(code[end - 1], b' ' | b'\t' | b'\r') {
        end -= 1;
    }
    &code[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ResolverMethodLength, "cops/graphql/resolver_method_length");
}
