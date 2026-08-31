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
        // task may be nested in expression statement
        return scan_back_for_desc(source, task);
    };
    if idx == 0 {
        return false;
    }
    let prev = siblings[idx - 1];
    is_desc_call(source, prev)
}

fn scan_back_for_desc(source: &SourceFile, task: Node<'_>) -> bool {
    let Some(mut node) = task.prev_named_sibling() else {
        return false;
    };
    loop {
        if is_desc_call(source, node) {
            return true;
        }
        // blank / comment-ish: keep looking one step
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
