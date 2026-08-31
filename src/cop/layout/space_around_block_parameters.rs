//! Layout/SpaceAroundBlockParameters.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceAroundBlockParameters;

fn strip_leading(bytes: &[u8], inner_s: usize, close: usize) -> usize {
    let mut e = inner_s;
    while e < close && matches!(bytes[e], b' ' | b'\t') { e += 1; }
    e
}

fn strip_trailing(bytes: &[u8], inner_s: usize, close: usize) -> usize {
    let mut s = close;
    while s > inner_s && matches!(bytes[s - 1], b' ' | b'\t') { s -= 1; }
    s
}

fn want_spaces(
    cop: &dyn Cop, source: &SourceFile, inner_s: usize, close: usize,
    after: Option<u8>, before: Option<u8>, sp_a: bool, sp_b: bool,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let msg = "Space missing inside block parameter pipes.";
    if !sp_a && after != Some(b'|') {
        report::insert_space(cop, source, inner_s, msg.into(), diagnostics, corrections, inner_s);
    }
    if !sp_b && before != Some(b'|') {
        report::insert_space(cop, source, close, msg.into(), diagnostics, corrections, close);
    }
}

fn no_spaces(
    cop: &dyn Cop, source: &SourceFile, bytes: &[u8], inner_s: usize, close: usize,
    sp_a: bool, sp_b: bool,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let msg = "Space present inside block parameter pipes.";
    if sp_a {
        report::report_fix(
            cop, source, inner_s, msg.into(), diagnostics, corrections,
            inner_s, strip_leading(bytes, inner_s, close), String::new(),
        );
    }
    if sp_b {
        report::report_fix(
            cop, source, close.saturating_sub(1), msg.into(), diagnostics, corrections,
            strip_trailing(bytes, inner_s, close), close, String::new(),
        );
    }
}

fn pipe_span(bytes: &[u8], node: Node<'_>) -> Option<(usize, usize)> {
    if bytes.get(node.start_byte()) != Some(&b'|') { return None; }
    if bytes.get(node.end_byte() - 1) != Some(&b'|') { return None; }
    Some((node.start_byte() + 1, node.end_byte() - 1))
}

impl Cop for SpaceAroundBlockParameters {
    fn name(&self) -> &'static str { "Layout/SpaceAroundBlockParameters" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["block_parameters"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyleInsidePipes", "no_space");
        let bytes = source.as_bytes();
        let Some((inner_s, close)) = pipe_span(bytes, node) else { return; };
        let after = bytes.get(inner_s).copied();
        let before = bytes.get(close.saturating_sub(1)).copied();
        let sp_a = matches!(after, Some(b' ') | Some(b'\t'));
        let sp_b = matches!(before, Some(b' ') | Some(b'\t'));
        if style == "space" {
            want_spaces(self, source, inner_s, close, after, before, sp_a, sp_b, diagnostics, &mut corrections);
        } else {
            no_spaces(self, source, bytes, inner_s, close, sp_a, sp_b, diagnostics, &mut corrections);
        }
    }
}
