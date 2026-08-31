//! Layout/AccessModifierIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct AccessModifierIndentation;

fn modifier_name<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    if node.kind() == "identifier" {
        Some(shared::node_bytes(source, node))
    } else {
        shared::call_method_name(source, node)
    }
}

fn is_modifier(name: &[u8]) -> bool {
    matches!(name, b"private" | b"protected" | b"public" | b"module_function")
}

fn enclosing_type(node: Node<'_>) -> Option<Node<'_>> {
    let mut p = node.parent();
    while let Some(n) = p {
        if matches!(n.kind(), "class" | "module" | "singleton_class") {
            return Some(n);
        }
        p = n.parent();
    }
    None
}

fn expected_col(style: &str, base: usize, width: usize) -> usize {
    if style == "outdent" { base } else { base + width }
}

fn report_modifier(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    name: &[u8],
    style: &str,
    expected: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mod_name = String::from_utf8_lossy(name);
    let style_word = if style == "outdent" { "Outdent" } else { "Indent" };
    report::fix_indent(
        cop,
        source,
        node.start_byte(),
        format!("{style_word} access modifiers like `{mod_name}`."),
        diagnostics,
        corrections,
        shared::line_indent(source, node.start_byte()),
        expected,
    );
}

impl Cop for AccessModifierIndentation {
    fn name(&self) -> &'static str {
        "Layout/AccessModifierIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "identifier"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "indent");
        let width = config.get_usize("IndentationWidth", 2);
        let Some(name) = modifier_name(source, node) else {
            return;
        };
        if !is_modifier(name) {
            return;
        }
        let Some(enclosing) = enclosing_type(node) else {
            return;
        };
        let expected = expected_col(style, shared::node_col(source, enclosing), width);
        if shared::node_col(source, node) == expected {
            return;
        }
        report_modifier(
            self,
            source,
            node,
            name,
            style,
            expected,
            diagnostics,
            &mut corrections,
        );
    }
}
