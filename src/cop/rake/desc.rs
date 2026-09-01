//! Rake/Desc — require `desc` before `task`.

use tree_sitter::{Node, Tree};

use super::RAKE_DEFAULT_INCLUDE;
use crate::cop::shared::{call_method_name, for_each_descendant};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct Desc;

impl Cop for Desc {
    fn name(&self) -> &'static str {
        "Rake/Desc"
    }

    fn default_include(&self) -> &'static [&'static str] {
        RAKE_DEFAULT_INCLUDE
    }

    fn uses_source_phase(&self) -> bool {
        true
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
        // Collect statements at each body level: look for task without preceding desc
        for_each_descendant(tree.root_node(), |node| {
            if !matches!(node.kind(), "call" | "command" | "command_call") {
                return;
            }
            if call_method_name(source, node) != Some(b"task") {
                return;
            }
            // RuboCop: default task needs no desc (`rake` with no args).
            if task_name_is_default(source, node) {
                return;
            }
            if preceded_by_desc(source, node) {
                return;
            }
            let (line, column) = source.offset_to_line_col(node.start_byte());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                "Describe the task with `desc` before the task definition.".into(),
            ));
        });
    }
}

fn preceded_by_desc(source: &SourceFile, task: Node<'_>) -> bool {
    let Some(parent) = task.parent() else {
        return false;
    };
    let mut cur = parent.walk();
    let siblings: Vec<_> = parent.named_children(&mut cur).collect();
    let Some(idx) = siblings.iter().position(|n| n.id() == task.id()) else {
        return scan_back_for_desc(source, task);
    };
    // Skip heredoc_body that follows `desc <<~DESC` / `desc <<-DESC`.
    let mut i = idx;
    while i > 0 {
        i -= 1;
        let prev = siblings[i];
        if prev.kind() == "heredoc_body" {
            continue;
        }
        return is_desc_call(source, prev);
    }
    false
}

fn scan_back_for_desc(source: &SourceFile, task: Node<'_>) -> bool {
    let Some(mut node) = task.prev_named_sibling() else {
        return false;
    };
    loop {
        if node.kind() == "heredoc_body" {
            match node.prev_named_sibling() {
                Some(p) => {
                    node = p;
                    continue;
                }
                None => return false,
            }
        }
        if is_desc_call(source, node) {
            return true;
        }
        if matches!(node.kind(), "comment") {
            match node.prev_named_sibling() {
                Some(p) => node = p,
                None => return false,
            }
            continue;
        }
        return false;
    }
}

fn is_desc_call(source: &SourceFile, node: Node<'_>) -> bool {
    let node = if node.kind() == "call" || node.kind() == "command" || node.kind() == "command_call" {
        node
    } else {
        // unwrap single-child wrappers
        let mut cur = node.walk();
        let kids: Vec<_> = node.named_children(&mut cur).collect();
        if kids.len() == 1 {
            kids[0]
        } else {
            return false;
        }
    };
    call_method_name(source, node) == Some(b"desc")
}

fn task_name_is_default(source: &SourceFile, task: Node<'_>) -> bool {
    let Some(args) = task.child_by_field_name("arguments") else {
        return false;
    };
    let mut cur = args.walk();
    args.named_children(&mut cur)
        .any(|child| child_is_default_name(source, child))
}

fn symbol_bare_name<'a>(source: &'a SourceFile, node: Node<'_>) -> &'a [u8] {
    let b = crate::cop::shared::node_bytes(source, node);
    b.strip_prefix(b":").unwrap_or(b)
}

fn child_is_default_name(source: &SourceFile, child: Node<'_>) -> bool {
    match child.kind() {
        "simple_symbol" | "hash_key_symbol" | "symbol" => {
            symbol_bare_name(source, child) == b"default"
        }
        "pair" => pair_key_is_default(source, child),
        _ => false,
    }
}

fn pair_key_is_default(source: &SourceFile, pair: Node<'_>) -> bool {
    let Some(key) = pair.child_by_field_name("key").or_else(|| {
        let mut c2 = pair.walk();
        pair.named_children(&mut c2).next()
    }) else {
        return false;
    };
    let name = symbol_bare_name(source, key);
    // `default:` hash key symbol may include trailing `:`
    let bare = name.strip_suffix(b":").unwrap_or(name);
    bare == b"default"
}
