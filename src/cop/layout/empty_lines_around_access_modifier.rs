//! Layout/EmptyLinesAroundAccessModifier.

use tree_sitter::{Node, Tree};

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct EmptyLinesAroundAccessModifier;

fn modifier_name<'a>(source: &'a SourceFile, n: Node<'_>) -> Option<&'a [u8]> {
    if n.kind() == "identifier" {
        Some(shared::node_bytes(source, n))
    } else {
        shared::call_method_name(source, n)
    }
}

fn is_modifier(name: &[u8]) -> bool {
    matches!(name, b"private" | b"protected" | b"public" | b"module_function")
}

fn bare_modifier(n: Node<'_>) -> bool {
    if n.kind() == "identifier" { return true; }
    match n.child_by_field_name("arguments") {
        Some(a) => a.named_child_count() == 0,
        None => true,
    }
}

fn maybe_before(
    cop: &dyn Cop, source: &SourceFile, line: usize, style: &str, before_ok: bool, name: &[u8],
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !(style == "around" || style == "only_before") || before_ok { return; }
    let modifier = String::from_utf8_lossy(name);
    report::insert_newline(
        cop, source, line,
        format!("Keep a blank line before and after `{modifier}`."),
        diagnostics, corrections,
    );
}

fn maybe_after(
    cop: &dyn Cop, source: &SourceFile, line: usize, style: &str, after_ok: bool, name: &[u8],
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !(style == "around" || style == "only_after") || after_ok { return; }
    let modifier = String::from_utf8_lossy(name);
    report::insert_newline(
        cop, source, line + 1,
        format!("Keep a blank line after `{modifier}`."),
        diagnostics, corrections,
    );
}

fn check_modifier(
    cop: &dyn Cop, source: &SourceFile, n: Node<'_>, style: &str,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !matches!(n.kind(), "call" | "command" | "identifier") { return; }
    let Some(name) = modifier_name(source, n) else { return; };
    if !is_modifier(name) || !bare_modifier(n) { return; }
    let line = shared::node_line(source, n);
    let before_ok = line <= 1 || shared::line_blank(source, line - 1);
    let after_ok = shared::line_blank(source, line + 1);
    maybe_before(cop, source, line, style, before_ok, name, diagnostics, corrections);
    maybe_after(cop, source, line, style, after_ok, name, diagnostics, corrections);
}

impl Cop for EmptyLinesAroundAccessModifier {
    fn name(&self) -> &'static str { "Layout/EmptyLinesAroundAccessModifier" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, tree: &Tree, code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = code_map;
        let style = config.get_str("EnforcedStyle", "around");
        shared::for_each_descendant(tree.root_node(), |n| {
            check_modifier(self, source, n, style, diagnostics, &mut corrections);
        });
    }
}
