//! Rake/ClassDefinitionInTask — no class/module inside task/namespace.

use tree_sitter::{Node, Tree};

use super::RAKE_DEFAULT_INCLUDE;
use crate::cop::shared::{call_method_name, for_each_descendant};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct ClassDefinitionInTask;

impl Cop for ClassDefinitionInTask {
    fn name(&self) -> &'static str {
        "Rake/ClassDefinitionInTask"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
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
        for_each_descendant(tree.root_node(), |node| {
            if !matches!(node.kind(), "class" | "module") {
                return;
            }
            if !inside_task_or_namespace(source, node) {
                return;
            }
            // Nested class inside another class already in task: only outermost
            if let Some(parent) = node.parent()
                && ancestor_is_classlike(parent)
            {
                return;
            }
            let kind = if node.kind() == "class" { "class" } else { "module" };
            let (line, column) = source.offset_to_line_col(node.start_byte());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!(
                    "Do not define a {kind} in rake task, because it will be defined to the top level."
                ),
            ));
        });
    }
}

fn ancestor_is_classlike(mut node: Node<'_>) -> bool {
    loop {
        if matches!(node.kind(), "class" | "module" | "singleton_class") {
            return true;
        }
        match node.parent() {
            Some(p) => node = p,
            None => return false,
        }
    }
}

fn inside_task_or_namespace(source: &SourceFile, mut node: Node<'_>) -> bool {
    while let Some(parent) = node.parent() {
        if is_task_block(source, parent) || is_task_call(source, parent) {
            return true;
        }
        node = parent;
    }
    false
}

fn is_task_block(source: &SourceFile, parent: Node<'_>) -> bool {
    matches!(parent.kind(), "block" | "do_block")
        && parent
            .parent()
            .is_some_and(|call| is_task_or_ns_call(source, call))
}

fn is_task_call(source: &SourceFile, parent: Node<'_>) -> bool {
    is_task_or_ns_call(source, parent) && has_block(parent)
}

fn is_task_or_ns_call(source: &SourceFile, call: Node<'_>) -> bool {
    matches!(call.kind(), "call" | "command" | "command_call")
        && matches!(call_method_name(source, call), Some(b"task") | Some(b"namespace"))
}

fn has_block(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|c| matches!(c.kind(), "block" | "do_block"))
}
