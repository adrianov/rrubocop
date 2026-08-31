//! Bundler/DuplicatedGem — flag repeated `gem 'name'` (simple non-conditional).

use std::collections::HashMap;

use tree_sitter::{Node, Tree};

use crate::cop::shared::{argument_nodes, call_method_name, for_each_descendant, node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct DuplicatedGem;

impl Cop for DuplicatedGem {
    fn name(&self) -> &'static str {
        "Bundler/DuplicatedGem"
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
        let mut first: HashMap<Vec<u8>, (usize, usize)> = HashMap::new();
        for_each_descendant(tree.root_node(), |node| {
            if !is_gem_call(source, node) {
                return;
            }
            // Skip gems inside if/unless/case (conditional duplicates allowed).
            if under_conditional(node) {
                return;
            }
            let Some(name) = gem_name(source, node) else {
                return;
            };
            let (line, column) = source.offset_to_line_col(node.start_byte());
            if let Some(&(first_line, _)) = first.get(&name) {
                let gem_name = String::from_utf8_lossy(&name);
                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    column,
                    format!(
                        "Gem `{gem_name}` requirements already given on line {first_line} of the Gemfile."
                    ),
                ));
            } else {
                first.insert(name, (line, column));
            }
        });
    }
}

fn is_gem_call(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(node.kind(), "call" | "command" | "command_call")
        && call_method_name(source, node) == Some(b"gem")
}

fn gem_name(source: &SourceFile, node: Node<'_>) -> Option<Vec<u8>> {
    let first = argument_nodes(node).into_iter().next()?;
    let bytes = node_bytes(source, first);
    Some(strip_quotes(bytes).to_vec())
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

fn under_conditional(mut node: Node<'_>) -> bool {
    while let Some(parent) = node.parent() {
        match parent.kind() {
            "if" | "unless" | "case" | "when" | "elsif" | "else" | "if_modifier" | "unless_modifier" => {
                return true;
            }
            _ => node = parent,
        }
    }
    false
}

#[allow(dead_code)]
fn _dbg(source: &SourceFile, node: Node<'_>) -> String {
    node_text(source, node)
}
