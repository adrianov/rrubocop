use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/Debugger — debugger entry points.
pub struct Debugger;

const DEFAULT_SPECS: &[&str] = &[
    "binding.irb",
    "Kernel.binding.irb",
    "byebug",
    "remote_byebug",
    "Kernel.byebug",
    "Kernel.remote_byebug",
    "page.save_and_open_page",
    "page.save_and_open_screenshot",
    "page.save_page",
    "page.save_screenshot",
    "save_and_open_page",
    "save_and_open_screenshot",
    "save_page",
    "save_screenshot",
    "binding.b",
    "binding.break",
    "Kernel.binding.b",
    "Kernel.binding.break",
    "binding.pry",
    "binding.remote_pry",
    "binding.pry_remote",
    "Kernel.binding.pry",
    "Kernel.binding.remote_pry",
    "Kernel.binding.pry_remote",
    "Pry.rescue",
    "pry",
    "debugger",
    "Kernel.debugger",
    "jard",
    "binding.console",
];

const DEFAULT_REQUIRES: &[&str] = &["debug/open", "debug/start"];

fn push_receiver<'a>(
    source: &SourceFile,
    parts: &mut Vec<String>,
    node: Node<'a>,
) -> Option<Node<'a>> {
    match call_receiver(node) {
        Some(r) if r.kind() == "call" => Some(r),
        Some(r) if r.kind() == "identifier" || r.kind() == "constant" => {
            parts.push(node_text(source, r));
            None
        }
        _ => None,
    }
}

fn call_chain(source: &SourceFile, mut node: Node<'_>) -> String {
    let mut parts = Vec::new();
    loop {
        let meth = call_method_name(source, node)
            .map(|m| String::from_utf8_lossy(m).into_owned())
            .unwrap_or_default();
        parts.push(meth);
        match push_receiver(source, &mut parts, node) {
            Some(next) => node = next,
            None => break,
        }
    }
    parts.reverse();
    parts.join(".")
}

fn is_receiver_use(source: &SourceFile, node: Node<'_>) -> bool {
    let bytes = source.as_bytes();
    node.end_byte() < bytes.len() && matches!(bytes[node.end_byte()], b'.' | b'&')
}

fn chain_matches(chain: &str, spec: &str) -> bool {
    *chain == *spec || (spec.contains('.') && chain.ends_with(spec))
}

fn require_arg(source: &SourceFile, node: Node<'_>) -> Option<String> {
    if call_method_name(source, node) != Some(b"require") || call_receiver(node).is_some() {
        return None;
    }
    let arg = argument_nodes(node).into_iter().next()?;
    (arg.kind() == "string").then(|| {
        node_text(source, arg)
            .trim_matches(|c| c == '\'' || c == '"')
            .to_string()
    })
}

fn check_require(source: &SourceFile, node: Node<'_>) -> Option<String> {
    let inner = require_arg(source, node)?;
    DEFAULT_REQUIRES
        .contains(&inner.as_str())
        .then(|| node_text(source, node))
}

fn matching_spec(source: &SourceFile, node: Node<'_>) -> Option<String> {
    let chain = call_chain(source, node);
    let leaf = call_method_name(source, node).unwrap_or(b"");
    for spec in DEFAULT_SPECS {
        let spec_leaf = spec.rsplit('.').next().unwrap_or(spec);
        if leaf != spec_leaf.as_bytes() || !chain_matches(&chain, spec) {
            continue;
        }
        if is_receiver_use(source, node) {
            return None;
        }
        return Some(node_text(source, node));
    }
    None
}

impl Cop for Debugger {
    fn name(&self) -> &'static str {
        "Lint/Debugger"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let src = check_require(source, node).or_else(|| matching_spec(source, node));
        let Some(src) = src else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Remove debugger entry point `{src}`."),
        ));
    }
}
