//! Bundler/DuplicatedGroup — flag repeated `group :a, :b`.

use std::collections::HashMap;

use tree_sitter::{Node, Tree};

use crate::cop::shared::{argument_nodes, call_method_name, for_each_descendant, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct DuplicatedGroup;

impl Cop for DuplicatedGroup {
    fn name(&self) -> &'static str {
        "Bundler/DuplicatedGroup"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*.gemfile", "**/Gemfile", "**/gems.rb"]
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
        for_each_descendant(tree.root_node(), |node| {
            visit_group(self, source, node, &mut seen, diagnostics);
        });
    }
}

fn visit_group(
    cop: &DuplicatedGroup,
    source: &SourceFile,
    node: Node<'_>,
    seen: &mut HashMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !matches!(node.kind(), "call" | "command" | "command_call") {
        return;
    }
    if call_method_name(source, node) != Some(b"group") {
        return;
    }
    let (key, display) = group_key(source, node);
    if key.is_empty() {
        return;
    }
    let (line, column) = source.offset_to_line_col(node.start_byte());
    if let Some(&first_line) = seen.get(&key) {
        diagnostics.push(cop.diagnostic(
            source,
            line,
            column,
            format!("Gem group `{display}` already defined on line {first_line} of the Gemfile."),
        ));
    } else {
        seen.insert(key, line);
    }
}

fn group_key(source: &SourceFile, node: Node<'_>) -> (String, String) {
    let mut names: Vec<String> = argument_nodes(node)
        .into_iter()
        .filter_map(|arg| arg_group_name(source, arg))
        .collect();
    names.sort();
    let display = names.join(", ");
    (names.join("\0"), display)
}

fn arg_group_name(source: &SourceFile, arg: Node<'_>) -> Option<String> {
    let b = node_bytes(source, arg);
    if arg.kind() == "simple_symbol" || arg.kind() == "symbol" || b.starts_with(b":") {
        Some(String::from_utf8_lossy(b.strip_prefix(b":").unwrap_or(b)).into_owned())
    } else if arg.kind() == "string" {
        Some(String::from_utf8_lossy(strip_quotes(b)).into_owned())
    } else {
        None
    }
}

fn strip_quotes(bytes: &[u8]) -> &[u8] {
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &bytes[1..bytes.len() - 1]
    } else {
        bytes
    }
}
