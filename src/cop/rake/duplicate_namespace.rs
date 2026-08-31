//! Rake/DuplicateNamespace — same namespace defined twice.

use std::collections::HashMap;

use tree_sitter::{Node, Tree};

use super::RAKE_DEFAULT_INCLUDE;
use crate::cop::shared::{argument_nodes, call_method_name, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct DuplicateNamespace;

impl Cop for DuplicateNamespace {
    fn name(&self) -> &'static str {
        "Rake/DuplicateNamespace"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn default_include(&self) -> &'static [&'static str] {
        RAKE_DEFAULT_INCLUDE
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        _code_map: &CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut seen: HashMap<String, usize> = HashMap::new();
        walk(source, tree.root_node(), &mut Vec::new(), &mut seen, self, diagnostics);
    }
}

fn walk(
    source: &SourceFile,
    node: Node<'_>,
    ns: &mut Vec<String>,
    seen: &mut HashMap<String, usize>,
    cop: &DuplicateNamespace,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if matches!(node.kind(), "call" | "command" | "command_call")
        && call_method_name(source, node) == Some(b"namespace")
        && let Some(name) = first_arg_name(source, node)
    {
        record_ns(source, node, ns, &name, seen, cop, diagnostics);
        ns.push(name);
        walk_children(source, node, ns, seen, cop, diagnostics);
        ns.pop();
        return;
    }
    walk_children(source, node, ns, seen, cop, diagnostics);
}

fn walk_children(
    source: &SourceFile,
    node: Node<'_>,
    ns: &mut Vec<String>,
    seen: &mut HashMap<String, usize>,
    cop: &DuplicateNamespace,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        walk(source, child, ns, seen, cop, diagnostics);
    }
}

fn record_ns(
    source: &SourceFile,
    node: Node<'_>,
    ns: &[String],
    name: &str,
    seen: &mut HashMap<String, usize>,
    cop: &DuplicateNamespace,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let full = if ns.is_empty() {
        name.to_string()
    } else {
        format!("{}:{name}", ns.join(":"))
    };
    let (line, column) = source.offset_to_line_col(node.start_byte());
    if let Some(&first) = seen.get(&full) {
        diagnostics.push(cop.diagnostic(
            source,
            line,
            column,
            format!("Namespace `{full}` is already defined at line {first}."),
        ));
    } else {
        seen.insert(full, line);
    }
}

fn first_arg_name(source: &SourceFile, node: Node<'_>) -> Option<String> {
    let first = argument_nodes(node).into_iter().next()?;
    Some(String::from_utf8_lossy(strip_literal(node_bytes(source, first))).into_owned())
}

fn strip_literal(b: &[u8]) -> &[u8] {
    let b = b.strip_prefix(b":").unwrap_or(b);
    if b.len() >= 2
        && ((b[0] == b'\'' && b[b.len() - 1] == b'\'')
            || (b[0] == b'"' && b[b.len() - 1] == b'"'))
    {
        &b[1..b.len() - 1]
    } else {
        b
    }
}
