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

fn bad_gap(bytes: &[u8], method_end: usize, first_start: usize) -> bool {
    let between = &bytes[method_end..first_start];
    let spaces = between.iter().filter(|&&b| b == b' ' || b == b'\t').count();
    !between.iter().any(|&b| b == b'\n') && spaces != 1
}

fn offense<'a>(
    source: &'a SourceFile,
    node: Node<'a>,
) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let method = method_node(node)?;
    let args = node.child_by_field_name("arguments")?;
    if bytes.get(args.start_byte()) == Some(&b'(') {
        return None;
    }
    let first = first_arg(args)?;
    bad_gap(bytes, method.end_byte(), first.start_byte())
        .then_some((method.end_byte(), first.start_byte()))
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
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some((start, end)) = offense(source, node) else {
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
