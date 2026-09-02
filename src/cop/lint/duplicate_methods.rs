use std::collections::HashMap;

use tree_sitter::Node;

use crate::cop::shared::node_text;
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
        let mut seen: HashMap<(String, bool), usize> = HashMap::new();
        let mut cur = body.walk();
        for child in body.named_children(&mut cur) {
            let Some((name, singleton, line)) = method_key(source, child) else {
                continue;
            };
            if let Some(prev) = seen.insert((name.clone(), singleton), line) {
                report_dup(self, source, child, &name, singleton, prev, line, diagnostics);
            }
        }
    }
}
