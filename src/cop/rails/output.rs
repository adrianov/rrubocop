//! Rails/Output — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, method_node, node_text, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct Output;

const METHODS: &[&[u8]] = &[b"puts", b"print", b"p", b"pp", b"pretty_print", b"ap"];

fn stdout_span(source: &SourceFile, node: Node<'_>) -> Option<(usize, usize)> {
    let method = call_method_name(source, node)?;
    if !METHODS.contains(&method) {
        return None;
    }
    if let Some(recv) = call_receiver(node) {
        let t = node_text(source, recv);
        if t != "Kernel" && t != "::Kernel" {
            return None;
        }
    }
    let meth = method_node(node).unwrap_or(node);
    let start = call_receiver(node)
        .map(|r| r.start_byte())
        .unwrap_or(meth.start_byte());
    Some((start, meth.end_byte()))
}

impl Cop for Output {
    fn name(&self) -> &'static str {
        "Rails/Output"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &[
            "**/app/**/*.rb",
            "**/config/**/*.rb",
            "db/**/*.rb",
            "**/lib/**/*.rb",
        ]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn safe_autocorrect(&self) -> bool {
        false
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
        let Some((start, end)) = stdout_span(source, node) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(start);
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            "Do not write to stdout. Use Rails's logger if you want to log.".to_string(),
        );
        if push_replace(
            &mut corrections,
            start,
            end,
            "Rails.logger.debug",
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}
