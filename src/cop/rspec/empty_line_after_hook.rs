//! RSpec/EmptyLineAfterHook — blank line after `before`/`after`/`around` blocks.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLineAfterHook;

const HOOKS: &[&[u8]] = &[b"before", b"after", b"around"];

fn hook_block(node: Node<'_>) -> Option<Node<'_>> {
    let p = node.parent()?;
    matches!(p.kind(), "block" | "do_block").then_some(p)
}

fn has_blank_or_end(source: &SourceFile, end_node: Node<'_>) -> bool {
    let end_line = source.offset_to_line_col(end_node.end_byte()).0;
    let Some(next_start) = source.line_start(end_line + 1) else {
        return true;
    };
    let bytes = source.as_bytes();
    let next_end = source.line_start(end_line + 2).unwrap_or(bytes.len());
    let next = std::str::from_utf8(&bytes[next_start..next_end])
        .unwrap_or("")
        .trim();
    next.is_empty() || next == "end"
}

impl Cop for EmptyLineAfterHook {
    fn name(&self) -> &'static str {
        "RSpec/EmptyLineAfterHook"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        if !HOOKS.contains(&method) {
            return;
        }
        let Some(end_node) = hook_block(node) else {
            return;
        };
        if has_blank_or_end(source, end_node) {
            return;
        }
        let insert_at = end_node.end_byte();
        let (line, col) = source.offset_to_line_col(insert_at);
        let name = std::str::from_utf8(method).unwrap_or("hook");
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            format!("Add an empty line after `{name}`."),
        );
        if push_replace(
            &mut corrections,
            insert_at,
            insert_at,
            "\n",
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}
