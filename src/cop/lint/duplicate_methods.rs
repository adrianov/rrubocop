use std::collections::HashMap;

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, is_const_named, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/DuplicateMethods — same method defined twice in a class/module.
pub struct DuplicateMethods;

fn method_key(source: &SourceFile, child: Node<'_>) -> Option<(String, bool, usize)> {
    if !matches!(child.kind(), "method" | "singleton_method") {
        return None;
    }
    let name_node = child.child_by_field_name("name")?;
    let name = node_text(source, name_node);
    let singleton = child.kind() == "singleton_method";
    let (line, _) = source.offset_to_line_col(child.start_byte());
    Some((name, singleton, line))
}

fn report_dup(
    cop: &DuplicateMethods,
    source: &SourceFile,
    child: Node<'_>,
    name: &str,
    singleton: bool,
    prev_line: usize,
    line: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let path = source.path_str();
    let display = if singleton {
        format!(".{name}")
    } else {
        format!("#{name}")
    };
    let (_, col) = source.offset_to_line_col(child.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!("Method `{display}` is defined at both {path}:{prev_line} and {path}:{line}."),
    ));
}

fn record_method(
    cop: &DuplicateMethods,
    source: &SourceFile,
    child: Node<'_>,
    seen: &mut HashMap<(String, bool), usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some((name, singleton, line)) = method_key(source, child) else {
        return;
    };
    if let Some(prev) = seen.insert((name.clone(), singleton), line) {
        report_dup(
            cop,
            source,
            child,
            &name,
            singleton,
            prev,
            line,
            diagnostics,
        );
    }
}

fn block_call(block: Node<'_>) -> Option<Node<'_>> {
    let p = block.parent()?;
    matches!(p.kind(), "call" | "command" | "command_call").then_some(p)
}

fn walk_named(
    cop: &DuplicateMethods,
    source: &SourceFile,
    node: Node<'_>,
    seen: &mut HashMap<(String, bool), usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut cur = node.walk();
    for child in node.named_children(&mut cur) {
        walk_body(cop, source, child, seen, diagnostics);
    }
}

/// RuboCop tracks defs in bare `class_eval`, `Const.class_eval`, and
/// `Class`/`Module.new` blocks; other blocks (`included`, `class_methods`,
/// `each`, `self.class_eval`, …) leave `parent_module_name` nil.
fn walk_block(
    cop: &DuplicateMethods,
    source: &SourceFile,
    block: Node<'_>,
    seen: &mut HashMap<(String, bool), usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(call) = block_call(block) else {
        return;
    };
    let Some(name) = call_method_name(source, call) else {
        return;
    };
    let recv = call_receiver(call);
    match name {
        b"class_eval" if recv.is_none() => {
            walk_named(cop, source, block, seen, diagnostics);
        }
        b"class_eval"
            if recv.is_some_and(|r| matches!(r.kind(), "constant" | "scope_resolution")) =>
        {
            walk_named(cop, source, block, &mut HashMap::new(), diagnostics);
        }
        b"new"
            if recv.is_some_and(|r| {
                is_const_named(source, r, b"Class") || is_const_named(source, r, b"Module")
            }) =>
        {
            walk_named(cop, source, block, &mut HashMap::new(), diagnostics);
        }
        _ => {}
    }
}

/// Walk body: `case`/`when` duplicates count; `if`/`unless` branches do not.
fn walk_body(
    cop: &DuplicateMethods,
    source: &SourceFile,
    node: Node<'_>,
    seen: &mut HashMap<(String, bool), usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if matches!(node.kind(), "method" | "singleton_method") {
        record_method(cop, source, node, seen, diagnostics);
        return;
    }
    if matches!(node.kind(), "if" | "unless") {
        return;
    }
    if matches!(node.kind(), "class" | "module" | "singleton_class") {
        return;
    }
    if matches!(node.kind(), "do_block" | "block" | "lambda") {
        walk_block(cop, source, node, seen, diagnostics);
        return;
    }
    walk_named(cop, source, node, seen, diagnostics);
}

impl Cop for DuplicateMethods {
    fn name(&self) -> &'static str {
        "Lint/DuplicateMethods"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class", "module", "singleton_class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(body) = node.child_by_field_name("body") else {
            return;
        };
        walk_body(self, source, body, &mut HashMap::new(), diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(DuplicateMethods, "cops/lint/duplicate_methods");
}
