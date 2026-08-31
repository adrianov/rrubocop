use tree_sitter::Node;

use crate::cop::shared::{call_method_name, node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/UselessAccessModifier — access modifier with no following methods.
pub struct UselessAccessModifier;

const MODS: &[&[u8]] = &[b"private", b"protected", b"public", b"module_function"];

fn is_mod_id(source: &SourceFile, node: Node<'_>) -> bool {
    node.kind() == "identifier" && MODS.contains(&node_bytes(source, node))
}

fn is_bare_mod_call(source: &SourceFile, node: Node<'_>) -> bool {
    node.kind() == "call"
        && matches!(
            call_method_name(source, node),
            Some(b"private" | b"protected" | b"public" | b"module_function")
        )
        && node.child_by_field_name("arguments").is_none()
}

fn is_modifier(source: &SourceFile, node: Node<'_>) -> bool {
    is_mod_id(source, node) || is_bare_mod_call(source, node)
}

fn mod_name(source: &SourceFile, node: Node<'_>) -> String {
    if node.kind() == "identifier" {
        return node_text(source, node);
    }
    call_method_name(source, node)
        .map(|m| String::from_utf8_lossy(m).into_owned())
        .unwrap_or_default()
}

fn has_following_method(source: &SourceFile, children: &[Node<'_>], from: usize) -> bool {
    for next in &children[from..] {
        if is_mod_id(source, *next) {
            return false;
        }
        if matches!(next.kind(), "method" | "singleton_method") {
            return true;
        }
    }
    false
}

fn report_useless(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    child: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = mod_name(source, child);
    let (line, col) = source.offset_to_line_col(child.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!("Useless `{name}` access modifier."),
    ));
}

fn scan_modifiers(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    children: &[Node<'_>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (i, child) in children.iter().enumerate() {
        if !is_modifier(source, *child) || has_following_method(source, children, i + 1) {
            continue;
        }
        report_useless(cop, source, *child, diagnostics);
    }
}

impl Cop for UselessAccessModifier {
    fn name(&self) -> &'static str {
        "Lint/UselessAccessModifier"
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
        let mut cur = body.walk();
        let children: Vec<_> = body.named_children(&mut cur).collect();
        scan_modifiers(self, source, &children, diagnostics);
    }
}
