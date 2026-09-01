//! Rails/RakeEnvironment — rake tasks should depend on `:environment`.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RakeEnvironment;

impl Cop for RakeEnvironment {
    fn name(&self) -> &'static str {
        "Rails/RakeEnvironment"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/Rakefile", "**/*.rake"]
    }

    fn default_exclude(&self) -> &'static [&'static str] {
        &["**/lib/capistrano/tasks/**/*.rake"]
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
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if call_receiver(node).is_some() {
            return;
        }
        if call_method_name(source, node) != Some(b"task") {
            return;
        }
        if !has_task_block(node) {
            return;
        }
        if task_name_is_default(source, node) {
            return;
        }
        if with_dependencies(node) {
            return;
        }
        let (line, column) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            "Include `:environment` task as a dependency for all Rake tasks.".into(),
        ));
    }
}

fn has_task_block(node: Node<'_>) -> bool {
    if node.child_by_field_name("block").is_some() {
        return true;
    }
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|c| matches!(c.kind(), "block" | "do_block"))
}

fn task_name_is_default(source: &SourceFile, task: Node<'_>) -> bool {
    let args = argument_nodes(task);
    let Some(first) = args.first() else {
        return false;
    };
    match first.kind() {
        "simple_symbol" | "symbol" | "string" | "string_content" => {
            symbol_or_string_is_default(source, *first)
        }
        "hash" => hash_key_is_default(source, *first),
        "pair" => pair_key_is_default(source, *first),
        _ => false,
    }
}

fn symbol_or_string_is_default(source: &SourceFile, node: Node<'_>) -> bool {
    let b = node_bytes(source, node);
    let name = b.strip_prefix(b":").unwrap_or(b);
    let name = name
        .strip_prefix(b"'")
        .or_else(|| name.strip_prefix(b"\""))
        .unwrap_or(name);
    let name = name
        .strip_suffix(b"'")
        .or_else(|| name.strip_suffix(b"\""))
        .unwrap_or(name);
    name == b"default"
}

fn pair_key_is_default(source: &SourceFile, pair: Node<'_>) -> bool {
    let Some(key) = pair.child_by_field_name("key").or_else(|| {
        let mut c = pair.walk();
        pair.named_children(&mut c).next()
    }) else {
        return false;
    };
    let b = node_bytes(source, key);
    let name = b.strip_prefix(b":").unwrap_or(b);
    let bare = name.strip_suffix(b":").unwrap_or(name);
    bare == b"default" || symbol_or_string_is_default(source, key)
}

fn hash_key_is_default(source: &SourceFile, hash: Node<'_>) -> bool {
    let mut cur = hash.walk();
    let pairs: Vec<_> = hash
        .named_children(&mut cur)
        .filter(|n| n.kind() == "pair")
        .collect();
    if pairs.len() != 1 {
        return false;
    }
    pair_key_is_default(source, pairs[0])
}

fn with_dependencies(node: Node<'_>) -> bool {
    let args = argument_nodes(node);
    let Some(first) = args.first() else {
        return false;
    };
    if is_hash_arg(*first) {
        return hash_style_deps(*first);
    }
    if let Some(second) = args.get(1) {
        if is_hash_arg(*second) {
            return hash_style_deps(*second);
        }
    }
    false
}

fn is_hash_arg(node: Node<'_>) -> bool {
    matches!(node.kind(), "hash" | "pair")
}

fn hash_style_deps(node: Node<'_>) -> bool {
    let pairs: Vec<Node<'_>> = if node.kind() == "pair" {
        vec![node]
    } else {
        let mut cur = node.walk();
        node.named_children(&mut cur)
            .filter(|n| n.kind() == "pair")
            .collect()
    };
    let Some(pair) = pairs.first() else {
        return false;
    };
    let Some(value) = pair.child_by_field_name("value").or_else(|| {
        let mut c = pair.walk();
        pair.named_children(&mut c).nth(1)
    }) else {
        return false;
    };
    if value.kind() == "array" {
        let mut c = value.walk();
        return value.named_children(&mut c).next().is_some();
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RakeEnvironment, "cops/rails/rake_environment");
}
