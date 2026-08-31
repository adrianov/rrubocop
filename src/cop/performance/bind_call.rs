//! Performance/BindCall — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BindCall;

fn bind_arg(source: &SourceFile, node: Node<'_>) -> Option<String> {
    if call_method_name(source, node) != Some(b"call") {
        return None;
    }
    let recv = call_receiver(node)?;
    if !matches!(recv.kind(), "call" | "command") {
        return None;
    }
    if call_method_name(source, recv) != Some(b"bind") {
        return None;
    }
    let args = argument_nodes(recv);
    (args.len() == 1).then(|| node_text(source, args[0]))
}

fn call_args_src(source: &SourceFile, node: Node<'_>) -> String {
    argument_nodes(node)
        .into_iter()
        .map(|a| node_text(source, a))
        .collect::<Vec<_>>()
        .join(", ")
}

fn offense_msg(source: &SourceFile, node: Node<'_>) -> Option<String> {
    let bind_arg = bind_arg(source, node)?;
    let args = call_args_src(source, node);
    let comma = if args.is_empty() { "" } else { ", " };
    Some(format!(
        "Use `bind_call({bind_arg}{comma}{args})` instead of `bind({bind_arg}).call({args})`."
    ))
}

impl Cop for BindCall {
    fn name(&self) -> &'static str {
        "Performance/BindCall"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(msg) = offense_msg(source, node) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, msg));
    }
}
