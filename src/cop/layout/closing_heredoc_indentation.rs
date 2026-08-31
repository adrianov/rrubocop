//! Layout/ClosingHeredocIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ClosingHeredocIndentation;

fn root_of(node: Node<'_>) -> Node<'_> {
    let mut x = node;
    while let Some(p) = x.parent() { x = p; }
    x
}

fn consider_end<'a>(
    source: &SourceFile, node: Node<'a>, after: usize, delim: &str, best: &mut Option<Node<'a>>,
) {
    if node.kind() != "heredoc_end" || node.start_byte() <= after { return; }
    if shared::node_text(source, node).trim() != delim { return; }
    let replace = match *best {
        None => true,
        Some(prev) => node.start_byte() < prev.start_byte(),
    };
    if replace { *best = Some(node); }
}

fn walk<'a>(
    source: &SourceFile, node: Node<'a>, after: usize, delim: &str, best: &mut Option<Node<'a>>,
) {
    consider_end(source, node, after, delim, best);
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        walk(source, child, after, delim, best);
    }
}

fn find_heredoc_end<'a>(
    source: &SourceFile, node: Node<'a>, after: usize, delim: &str,
) -> Option<Node<'a>> {
    let mut best = None;
    walk(source, node, after, delim, &mut best);
    best
}

fn delim_of(open_text: &str) -> &str {
    open_text
        .trim_start_matches('<')
        .trim_start_matches(['~', '-', '\'', '"'])
        .trim_matches(['\'', '"'])
}

impl Cop for ClosingHeredocIndentation {
    fn name(&self) -> &'static str { "Layout/ClosingHeredocIndentation" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["heredoc_beginning"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let open_text = shared::node_text(source, node);
        let delim = delim_of(&open_text);
        let open_col = shared::node_col(source, node);
        let Some(end_n) = find_heredoc_end(source, root_of(node), node.start_byte(), delim) else {
            return;
        };
        if shared::node_col(source, end_n) == open_col { return; }
        report::fix_indent(
            self, source, end_n.start_byte(),
            format!("`{delim}` is not aligned with `{open_text}`."),
            diagnostics, &mut corrections,
            shared::line_indent(source, end_n.start_byte()), open_col,
        );
    }
}
