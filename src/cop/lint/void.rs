use tree_sitter::Node;

use crate::cop::shared::{call_method_name, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/Void — literals/ops/vars in void context.
pub struct Void;

fn void_msg(source: &SourceFile, stmt: Node<'_>) -> Option<String> {
    match stmt.kind() {
        "integer" | "float" | "string" | "simple_symbol" | "regex" | "array" | "hash" | "true"
        | "false" | "nil" => Some(format!(
            "Literal `{}` used in void context.",
            node_text(source, stmt)
        )),
        "self" => Some("`self` used in void context.".to_string()),
        "constant" => Some(format!(
            "Constant `{}` used in void context.",
            node_text(source, stmt)
        )),
        "identifier" => Some(format!(
            "Variable `{}` used in void context.",
            node_text(source, stmt)
        )),
        "binary" => {
            let op = stmt
                .child_by_field_name("operator")
                .map(|o| node_text(source, o))
                .unwrap_or_default();
            Some(format!("Operator `{op}` used in void context."))
        }
        _ => None,
    }
}

fn check_void_stmts(
    source: &SourceFile,
    stmts: &[Node<'_>],
    cop: &Void,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in stmts.iter().take(stmts.len().saturating_sub(1)) {
        let Some(msg) = void_msg(source, *stmt) else {
            continue;
        };
        let (line, col) = source.offset_to_line_col(stmt.start_byte());
        diagnostics.push(cop.diagnostic(source, line, col, msg));
    }
}

fn nonmutating_suggest(meth: &[u8]) -> Option<&'static str> {
    match meth {
        b"map" | b"collect" => Some("each"),
        b"reverse" => Some("reverse!"),
        b"sort" => Some("sort!"),
        _ => None,
    }
}

fn check_nonmutating(
    source: &SourceFile,
    stmts: &[Node<'_>],
    cop: &Void,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in stmts.iter().take(stmts.len().saturating_sub(1)) {
        if stmt.kind() != "call" {
            continue;
        }
        let Some(meth) = call_method_name(source, *stmt) else {
            continue;
        };
        let Some(sug) = nonmutating_suggest(meth) else {
            continue;
        };
        let m = String::from_utf8_lossy(meth);
        let (line, col) = source.offset_to_line_col(stmt.start_byte());
        diagnostics.push(cop.diagnostic(
            source,
            line,
            col,
            format!("Method `#{m}` used in void context. Did you mean `#{sug}`?"),
        ));
    }
}

impl Cop for Void {
    fn name(&self) -> &'static str {
        "Lint/Void"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["body_statement", "block_body"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut cur = node.walk();
        let stmts: Vec<_> = node.named_children(&mut cur).collect();
        check_void_stmts(source, &stmts, self, diagnostics);
        check_nonmutating(source, &stmts, self, diagnostics);
    }
}
