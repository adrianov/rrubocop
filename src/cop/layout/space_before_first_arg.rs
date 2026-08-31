//! Layout/SpaceBeforeFirstArg.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceBeforeFirstArg;

fn method_node<'a>(node: Node<'a>) -> Option<Node<'a>> {
    node.child_by_field_name("method").or_else(|| node.child_by_field_name("name"))
}

fn first_arg<'a>(args: Node<'a>) -> Option<Node<'a>> {
    let mut cur = args.walk();
    args.named_children(&mut cur).next()
}

fn bad_gap(bytes: &[u8], method_end: usize, first_start: usize) -> bool {
    let between = &bytes[method_end..first_start];
    let spaces = between.iter().filter(|&&b| b == b' ' || b == b'\t').count();
    let has_nl = between.iter().any(|&b| b == b'\n');
    !has_nl && spaces != 1
}

impl Cop for SpaceBeforeFirstArg {
    fn name(&self) -> &'static str { "Layout/SpaceBeforeFirstArg" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["call", "command", "command_call"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        let bytes = source.as_bytes();
        let Some(method) = method_node(node) else { return; };
        let Some(args) = node.child_by_field_name("arguments") else { return; };
        if bytes.get(args.start_byte()) == Some(&b'(') { return; }
        let Some(first) = first_arg(args) else { return; };
        if !bad_gap(bytes, method.end_byte(), first.start_byte()) { return; }
        report::report_fix(
            self, source, method.end_byte(),
            "Put one space between the method name and the first argument.".into(),
            diagnostics, &mut corrections,
            method.end_byte(), first.start_byte(), " ".into(),
        );
    }
}
