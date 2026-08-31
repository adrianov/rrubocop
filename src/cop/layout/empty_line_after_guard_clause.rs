//! Layout/EmptyLineAfterGuardClause.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLineAfterGuardClause;

fn modifier_is_guard(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(body) = node.child_by_field_name("body") else { return false; };
    let k = body.kind();
    matches!(k, "return" | "break" | "next" | "call" | "command" | "raise")
        || shared::call_method_name(source, body) == Some(b"raise")
        || shared::node_bytes(source, body).starts_with(b"raise")
}

fn is_guard(source: &SourceFile, node: Node<'_>) -> bool {
    match node.kind() {
        "if_modifier" | "unless_modifier" => modifier_is_guard(source, node),
        _ => false,
    }
}

fn next_is_kw(bytes: &[u8], ls: usize) -> bool {
    let mut i = ls;
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') { i += 1; }
    let rest = &bytes[i..];
    rest.starts_with(b"end") || rest.starts_with(b"rescue")
        || rest.starts_with(b"ensure") || rest.starts_with(b"else")
        || rest.starts_with(b"elsif") || rest.starts_with(b"when")
        || rest.starts_with(b"#")
}

impl Cop for EmptyLineAfterGuardClause {
    fn name(&self) -> &'static str { "Layout/EmptyLineAfterGuardClause" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if_modifier", "unless_modifier", "return", "break", "next", "raise"]
    }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        if !is_guard(source, node) { return; }
        let (end_line, _) = source.offset_to_line_col(node.end_byte().saturating_sub(1));
        let next = end_line + 1;
        if source.line_start(next).is_none() || shared::line_blank(source, next) { return; }
        if let Some(ls) = source.line_start(next) {
            if next_is_kw(source.as_bytes(), ls) { return; }
        }
        report::insert_newline(
            self, source, next, "Add empty line after guard clause.".into(),
            diagnostics, &mut corrections,
        );
    }
}
