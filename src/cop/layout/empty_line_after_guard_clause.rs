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
    let Some(body) = node.child_by_field_name("body") else {
        return false;
    };
    let k = body.kind();
    // Only `return`/`break`/`next`/`raise` … if/unless — not arbitrary modifiers.
    matches!(k, "return" | "break" | "next")
        || shared::call_method_name(source, body) == Some(b"raise")
        || shared::node_bytes(source, body).starts_with(b"raise")
}

fn is_guard_node(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(node.kind(), "if_modifier" | "unless_modifier") && modifier_is_guard(source, node)
}

fn next_is_kw_or_comment(bytes: &[u8], ls: usize) -> bool {
    let mut i = ls;
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
        i += 1;
    }
    let rest = &bytes[i..];
    rest.starts_with(b"end")
        || rest.starts_with(b"rescue")
        || rest.starts_with(b"ensure")
        || rest.starts_with(b"else")
        || rest.starts_with(b"elsif")
        || rest.starts_with(b"when")
        || rest.starts_with(b"#")
}

fn line_starts_with_guard(source: &SourceFile, line: usize) -> bool {
    let Some(ls) = source.line_start(line) else {
        return false;
    };
    let bytes = source.as_bytes();
    let mut i = ls;
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
        i += 1;
    }
    let rest = &bytes[i..];
    rest.starts_with(b"return ")
        || rest.starts_with(b"return\t")
        || rest.starts_with(b"break ")
        || rest.starts_with(b"next ")
        || rest.starts_with(b"raise ")
        || rest.starts_with(b"raise(")
}

impl Cop for EmptyLineAfterGuardClause {
    fn name(&self) -> &'static str {
        "Layout/EmptyLineAfterGuardClause"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if_modifier", "unless_modifier"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        if !is_guard_node(source, node) {
            return;
        }
        let (end_line, _) = source.offset_to_line_col(node.end_byte().saturating_sub(1));
        let next = end_line + 1;
        if source.line_start(next).is_none() || shared::line_blank(source, next) {
            return;
        }
        if let Some(ls) = source.line_start(next) {
            if next_is_kw_or_comment(source.as_bytes(), ls) {
                return;
            }
        }
        // Consecutive guard clauses need no blank between them.
        if line_starts_with_guard(source, next) {
            return;
        }
        report::insert_newline(
            self,
            source,
            next,
            "Add empty line after guard clause.".into(),
            diagnostics,
            &mut corrections,
        );
    }
}
