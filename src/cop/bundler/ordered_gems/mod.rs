//! Bundler/OrderedGems — alphabetical gem order within Gemfile sections.

mod name;

use tree_sitter::{Node, Tree};

use crate::cop::shared::call_method_name;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct OrderedGems;

impl Cop for OrderedGems {
    fn name(&self) -> &'static str {
        "Bundler/OrderedGems"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*.gemfile", "**/Gemfile", "**/gems.rb"]
    }

    fn uses_source_phase(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        _code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let treat_comments = config.get_bool("TreatCommentsAsGroupSeparators", true);
        let consider_punct = config.get_bool("ConsiderPunctuation", false);
        let mut gems = collect_gem_calls(tree.root_node(), source);
        gems.sort_by_key(|n| n.start_byte());
        for win in gems.windows(2) {
            check_pair(
                self,
                source,
                win[0],
                win[1],
                treat_comments,
                consider_punct,
                diagnostics,
            );
        }
    }
}

fn collect_gem_calls<'a>(root: Node<'a>, source: &SourceFile) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    fn walk<'a>(node: Node<'a>, source: &SourceFile, out: &mut Vec<Node<'a>>) {
        if is_gem_call(source, node) {
            out.push(node);
        }
        let mut cur = node.walk();
        for child in node.children(&mut cur) {
            walk(child, source, out);
        }
    }
    walk(root, source, &mut out);
    out
}

fn check_pair(
    cop: &OrderedGems,
    source: &SourceFile,
    previous: Node<'_>,
    current: Node<'_>,
    treat_comments: bool,
    consider_punct: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !consecutive_gems(source, previous, current, treat_comments) {
        return;
    }
    let (Some(prev_name), Some(curr_name)) = (
        name::gem_name(source, previous),
        name::gem_name(source, current),
    ) else {
        return;
    };
    let prev_key = sort_key(&prev_name, consider_punct);
    let curr_key = sort_key(&curr_name, consider_punct);
    if curr_key >= prev_key {
        return;
    }
    let (line, column) = source.offset_to_line_col(current.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        column,
        format!(
            "Gems should be sorted in an alphabetical order within their section of the Gemfile. Gem `{curr_name}` should appear before `{prev_name}`."
        ),
    ));
}

fn consecutive_gems(
    source: &SourceFile,
    previous: Node<'_>,
    current: Node<'_>,
    treat_comments: bool,
) -> bool {
    let prev_last = previous.end_position().row;
    let orig_curr = current.start_position().row;
    let curr_first = if treat_comments {
        orig_curr
    } else {
        first_line_with_comments(source, orig_curr)
    };
    if prev_last + 1 != curr_first {
        return false;
    }
    for row in (prev_last + 1)..orig_curr {
        let Some(text) = source.line_text(row + 1) else {
            return false;
        };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return false;
        }
        if treat_comments || !trimmed.starts_with('#') {
            return false;
        }
    }
    true
}

fn first_line_with_comments(source: &SourceFile, mut row: usize) -> usize {
    while row > 0 {
        let Some(text) = source.line_text(row) else {
            break;
        };
        if text.trim_start().starts_with('#') {
            row -= 1;
        } else {
            break;
        }
    }
    row
}

fn sort_key(name: &str, consider_punctuation: bool) -> String {
    if consider_punctuation {
        return name.to_ascii_lowercase();
    }
    name.chars()
        .filter(|&c| c != '-' && c != '_')
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn is_gem_call(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(node.kind(), "call" | "command" | "command_call")
        && call_method_name(source, node) == Some(b"gem")
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(OrderedGems, "cops/bundler/ordered_gems");
}
