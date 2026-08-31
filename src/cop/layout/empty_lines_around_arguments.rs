//! Layout/EmptyLinesAroundArguments.

use tree_sitter::Node;

use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLinesAroundArguments;

fn is_multiline(source: &SourceFile, node: Node<'_>) -> bool {
    let start = shared::node_line(source, node);
    let (end, _) = source.offset_to_line_col(node.end_byte().saturating_sub(1));
    start != end
}

fn remove_blank(
    cop: &dyn Cop, source: &SourceFile, line: usize, at: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (l, c) = source.offset_to_line_col(at);
    let mut diag = cop.diagnostic(source, l, c, "Empty line detected around arguments.".into());
    if let Some(corr) = corrections {
        if let Some(s) = source.line_start(line) {
            let e = source.line_start(line + 1).unwrap_or(s);
            corr.push(Correction {
                start: s, end: e, replacement: String::new(),
                cop_name: cop.name(), cop_index: 0,
            });
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

fn check_pair(
    cop: &dyn Cop, source: &SourceFile, a: Node<'_>, b: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let la = shared::node_line(source, a);
    let lb = shared::node_line(source, b);
    if lb <= la + 1 { return; }
    for line in la + 1..lb {
        if shared::line_blank(source, line) {
            remove_blank(cop, source, line, b.start_byte(), diagnostics, corrections);
        }
    }
}

impl Cop for EmptyLinesAroundArguments {
    fn name(&self) -> &'static str { "Layout/EmptyLinesAroundArguments" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["argument_list", "command_argument_list"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        if !is_multiline(source, node) { return; }
        let mut cur = node.walk();
        let args: Vec<_> = node.named_children(&mut cur).collect();
        for w in args.windows(2) {
            check_pair(self, source, w[0], w[1], diagnostics, &mut corrections);
        }
    }
}
