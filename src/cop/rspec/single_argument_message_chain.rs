//! RSpec/SingleArgumentMessageChain — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, method_node, node_text, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SingleArgumentMessageChain;

fn chain_prefer(method: &[u8]) -> Option<&'static str> {
    match method {
        b"receive_message_chain" => Some("receive"),
        b"stub_chain" => Some("stub"),
        _ => None,
    }
}

fn named_count(node: Node<'_>, kind: Option<&str>) -> usize {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .filter(|n| kind.is_none_or(|k| n.kind() == k))
        .count()
}

fn single_message_arg(source: &SourceFile, arg: Node<'_>) -> bool {
    if matches!(arg.kind(), "string" | "string_content" | "interpolated_string")
        && node_text(source, arg).contains('.')
    {
        return false;
    }
    match arg.kind() {
        "array" => named_count(arg, None) == 1,
        "hash" => named_count(arg, Some("pair")) == 1,
        _ => true,
    }
}

fn single_chain_arg<'a, 't>(
    source: &'a SourceFile,
    node: Node<'t>,
) -> Option<(&'static str, &'a [u8], Node<'t>)> {
    let method = call_method_name(source, node)?;
    let prefer = chain_prefer(method)?;
    let args = argument_nodes(node);
    if args.len() != 1 || !single_message_arg(source, args[0]) {
        return None;
    }
    Some((prefer, method, args[0]))
}

fn unwrap_array_arg(
    source: &SourceFile,
    arg: Node<'_>,
    corrections: &mut Option<&mut Vec<Correction>>,
    cop_name: &'static str,
) {
    if arg.kind() != "array" {
        return;
    }
    let mut cur = arg.walk();
    if let Some(inner) = arg.named_children(&mut cur).next() {
        push_replace(
            corrections,
            arg.start_byte(),
            arg.end_byte(),
            node_text(source, inner),
            cop_name,
        );
    }
}

fn report_chain(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    prefer: &str,
    method: &[u8],
    arg: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let current = std::str::from_utf8(method).unwrap_or("");
    let meth = method_node(node).unwrap_or(node);
    let (line, col) = source.offset_to_line_col(meth.start_byte());
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        format!("Use `{prefer}` instead of calling `{current}` with a single argument."),
    );
    if push_replace(
        corrections,
        meth.start_byte(),
        meth.end_byte(),
        prefer,
        cop.name(),
    ) {
        unwrap_array_arg(source, arg, corrections, cop.name());
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

impl Cop for SingleArgumentMessageChain {
    fn name(&self) -> &'static str {
        "RSpec/SingleArgumentMessageChain"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["array", "call", "string", "symbol", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some((prefer, method, arg)) = single_chain_arg(source, node) else {
            return;
        };
        report_chain(
            self,
            source,
            node,
            prefer,
            method,
            arg,
            diagnostics,
            &mut corrections,
        );
    }
}
