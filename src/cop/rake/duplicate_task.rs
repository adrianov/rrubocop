//! Rake/DuplicateTask — same task name defined twice.

use std::collections::HashMap;

use tree_sitter::{Node, Tree};

use super::RAKE_DEFAULT_INCLUDE;
use crate::cop::shared::{argument_nodes, call_method_name, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct DuplicateTask;

impl Cop for DuplicateTask {
    fn name(&self) -> &'static str {
        "Rake/DuplicateTask"
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
    ns: &mut Vec<Option<String>>,
    seen: &mut HashMap<String, usize>,
    cop: &DuplicateTask,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if matches!(node.kind(), "call" | "command" | "command_call") {
        match call_method_name(source, node) {
            Some(b"namespace") if has_block(node) => {
                ns.push(extract_name(source, node));
                walk_children(source, node, ns, seen, cop, diagnostics);
                ns.pop();
                return;
            }
            Some(b"task") => record_task(source, node, ns, seen, cop, diagnostics),
            _ => {}
        }
    }
    walk_children(source, node, ns, seen, cop, diagnostics);
}

fn walk_children(
    source: &SourceFile,
    node: Node<'_>,
    ns: &mut Vec<Option<String>>,
    seen: &mut HashMap<String, usize>,
    cop: &DuplicateTask,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        walk(source, child, ns, seen, cop, diagnostics);
    }
}

fn record_task(
    source: &SourceFile,
    node: Node<'_>,
    ns: &[Option<String>],
    seen: &mut HashMap<String, usize>,
    cop: &DuplicateTask,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(task) = extract_name(source, node) else {
        return;
    };
    let Some(full) = full_name(ns, &task) else {
        return;
    };
    let (line, column) = source.offset_to_line_col(node.start_byte());
    if let Some(&first) = seen.get(&full) {
        diagnostics.push(cop.diagnostic(
            source,
            line,
            column,
            format!("Task `{full}` is already defined at line {first}."),
        ));
    } else {
        seen.insert(full, line);
    }
}

fn full_name(ns: &[Option<String>], task: &str) -> Option<String> {
    let parts: Option<Vec<&str>> = ns.iter().map(|n| n.as_deref()).collect();
    let parts = parts?;
    if parts.is_empty() {
        Some(task.to_string())
    } else {
        Some(format!("{}:{task}", parts.join(":")))
    }
}

fn extract_name(source: &SourceFile, node: Node<'_>) -> Option<String> {
    let first = *argument_nodes(node).first()?;
    match first.kind() {
        "string" | "simple_symbol" | "symbol" | "identifier" => {
            Some(String::from_utf8_lossy(strip_literal(node_bytes(source, first))).into_owned())
        }
        "pair" | "hash" => hash_task_name(source, first),
        _ => None,
    }
}

fn hash_task_name(source: &SourceFile, node: Node<'_>) -> Option<String> {
    let mut cur = node.walk();
    for child in node.named_children(&mut cur) {
        if child.kind() == "pair"
            && let Some(key) = child.child_by_field_name("key")
        {
            return Some(String::from_utf8_lossy(strip_literal(node_bytes(source, key))).into_owned());
        }
    }
    None
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

fn has_block(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|c| matches!(c.kind(), "block" | "do_block"))
}
