//! Rake/MethodDefinitionInTask — no `def` inside task/namespace.

use tree_sitter::{Node, Tree};

use super::RAKE_DEFAULT_INCLUDE;
use crate::cop::shared::call_method_name;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct MethodDefinitionInTask;

impl Cop for MethodDefinitionInTask {
    fn name(&self) -> &'static str {
        "Rake/MethodDefinitionInTask"
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
        walk(source, tree.root_node(), false, self, diagnostics);
    }
}

fn walk(
    source: &SourceFile,
    node: Node<'_>,
    in_task: bool,
    cop: &MethodDefinitionInTask,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let next_in_task = in_task || enters_task(source, node);
    if in_task && matches!(node.kind(), "method" | "singleton_method") {
        let (line, column) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(cop.diagnostic(
            source,
            line,
            column,
            "Do not define a method in rake task, because it will be defined to the top level."
                .into(),
        ));
    }
    // Don't flag methods inside classes defined in tasks (class cop covers that)
    if in_task && matches!(node.kind(), "class" | "module" | "singleton_class") {
        return;
    }
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        walk(source, child, next_in_task, cop, diagnostics);
    }
}

fn enters_task(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(node.kind(), "call" | "command" | "command_call")
        && matches!(call_method_name(source, node), Some(b"task") | Some(b"namespace"))
        && has_block(node)
}

fn has_block(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|c| matches!(c.kind(), "block" | "do_block"))
}
