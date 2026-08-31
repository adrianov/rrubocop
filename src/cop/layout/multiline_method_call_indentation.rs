//! Layout/MultilineMethodCallIndentation.

use tree_sitter::{Node, Tree};

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct MultilineMethodCallIndentation;

fn expected_col(source: &SourceFile, recv: Node<'_>, style: &str, width: usize) -> usize {
    if style == "indented" || style == "indented_relative_to_receiver" {
        shared::line_indent(source, recv.start_byte()) + width
    } else {
        shared::node_col(source, recv)
    }
}

fn check_call(
    cop: &dyn Cop, source: &SourceFile, n: Node<'_>, style: &str, width: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if n.kind() != "call" { return; }
    let Some(recv) = n.child_by_field_name("receiver") else { return; };
    let Some(method) = n.child_by_field_name("method") else { return; };
    if shared::node_line(source, recv) == shared::node_line(source, method) { return; }
    let expected = expected_col(source, recv, style, width);
    let actual = shared::line_indent(source, method.start_byte());
    if actual == expected { return; }
    report::fix_indent(
        cop, source, method.start_byte(),
        format!("Align method call receivers and their chained calls consistently (expected column {expected})."),
        diagnostics, corrections, actual, expected,
    );
}

impl Cop for MultilineMethodCallIndentation {
    fn name(&self) -> &'static str { "Layout/MultilineMethodCallIndentation" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, tree: &Tree, code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = code_map;
        let width = config.get_usize("IndentationWidth", 2);
        let style = config.get_str("EnforcedStyle", "aligned");
        shared::for_each_descendant(tree.root_node(), |n| {
            check_call(self, source, n, style, width, diagnostics, &mut corrections);
        });
    }
}
