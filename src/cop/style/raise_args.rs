//! Style/RaiseArgs — raise args style.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RaiseArgs;

impl Cop for RaiseArgs {
    fn name(&self) -> &'static str {
        "Style/RaiseArgs"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(msg) = raise_style_msg(source, node, config) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, msg));
    }
}

fn raise_style_msg(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> Option<String> {
    if !is_raise_call(source, node) {
        return None;
    }
    let style = config.get_str("EnforcedStyle", "exploded");
    let args = argument_nodes(node);
    if args.is_empty() {
        return None;
    }
    style_message(style, &args, source)
}

fn is_raise_call(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(call_method_name(source, node), Some(b"raise" | b"fail"))
}

fn style_message(style: &str, args: &[Node<'_>], source: &SourceFile) -> Option<String> {
    let compact = is_compact_raise(args, source);
    let exploded = args.len() >= 2;
    if style == "exploded" && compact {
        Some("Provide an exception class and message as arguments to `raise`.".to_string())
    } else if style == "compact" && exploded {
        Some("Provide an exception object as an argument to `raise`.".to_string())
    } else {
        None
    }
}

fn is_compact_raise(args: &[Node<'_>], source: &SourceFile) -> bool {
    args.len() == 1
        && args[0].kind() == "call"
        && call_method_name(source, args[0]) == Some(b"new")
}
