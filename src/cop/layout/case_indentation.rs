//! Layout/CaseIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CaseIndentation;

fn expected_col(style: &str, case_col: usize, width: usize) -> usize {
    if style == "case" { case_col } else { case_col + width }
}

impl Cop for CaseIndentation {
    fn name(&self) -> &'static str { "Layout/CaseIndentation" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["when", "else"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let width = config.get_usize("IndentationWidth", 2);
        let style = config.get_str("EnforcedStyle", "case");
        let Some(parent) = node.parent() else { return; };
        if parent.kind() != "case" && parent.kind() != "case_match" { return; }
        let expected = expected_col(style, shared::node_col(source, parent), width);
        let actual = shared::node_col(source, node);
        if actual == expected { return; }
        let branch = node.kind();
        report::fix_indent(
            self, source, node.start_byte(),
            format!("Indent `{branch}` one step more than `case`."),
            diagnostics, &mut corrections,
            shared::line_indent(source, node.start_byte()), expected,
        );
    }
}
