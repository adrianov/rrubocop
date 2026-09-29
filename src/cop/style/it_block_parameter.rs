//! Style/ItBlockParameter — `it` vs `_1` / named block args (Ruby 3.4+).

mod find;
mod shadow;

use tree_sitter::Node;

use crate::cop::shared::{node_bytes, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
use find::{
    block_body, explicit_params, host_node, idents, is_it_block, num_idents, plain_arg, single_line,
};

pub struct ItBlockParameter;

const USE_IT: &str = "Use `it` block parameter.";
const AVOID_IT: &str = "Avoid using `it` block parameter.";
const AVOID_MULTI: &str = "Avoid using `it` block parameter for multi-line blocks.";

fn ruby34(config: &CopConfig) -> bool {
    (config.get_f64("TargetRubyVersion", 3.4) * 10.0).round() >= 34.0
}

fn first_line_len(source: &SourceFile, node: Node<'_>) -> usize {
    let start = node.start_byte();
    let bytes = source.as_bytes();
    let mut i = start;
    let end = node.end_byte();
    while i < end && i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    (i - start).max(1)
}

fn push_at(
    cop: &ItBlockParameter,
    source: &SourceFile,
    node: Node<'_>,
    message: &str,
    correctable: bool,
    len: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, col) = source.offset_to_line_col(node.start_byte());
    let mut diag = cop.diagnostic(source, line, col, message.to_string());
    diag.correctable = correctable;
    diag.highlight_length = len;
    diagnostics.push(diag);
}

fn hspace(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

fn skip_hspace(bytes: &[u8], mut i: usize) -> usize {
    while bytes.get(i).is_some_and(|b| hspace(*b)) {
        i += 1;
    }
    i
}

fn trim_hspace(bytes: &[u8], mut start: usize) -> usize {
    while start > 0 && hspace(bytes[start - 1]) {
        start -= 1;
    }
    start
}

/// Range covering `|arg|`. On `{ |arg| x` the following space goes too.
/// On `do |arg|` / `{ |arg|` before a newline, the space before `|` and any
/// trailing horizontal whitespace go too, so the line does not end with a space.
fn param_span(source: &SourceFile, params: Node<'_>) -> (usize, usize) {
    let start = params.start_byte();
    let end = params.end_byte();
    let bytes = source.as_bytes();
    let ws_end = skip_hspace(bytes, end);
    if ws_end == bytes.len() || matches!(bytes.get(ws_end), Some(&b'\n' | &b'\r')) {
        return (trim_hspace(bytes, start), ws_end);
    }
    if start > 0 && bytes[start - 1] == b' ' && bytes.get(end) == Some(&b' ') {
        return (start, end + 1);
    }
    (start, end)
}

fn report_it(
    cop: &ItBlockParameter,
    source: &SourceFile,
    block: Node<'_>,
    style: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let body = block_body(block);
    match style {
        "allow_single_line" => {
            // `{ it }` is one line even when the receiver starts on the line above.
            if single_line(source, block) {
                return;
            }
            let host = host_node(block);
            push_at(
                cop,
                source,
                host,
                AVOID_MULTI,
                false,
                first_line_len(source, host),
                diagnostics,
            );
        }
        "disallow" => {
            let Some(body) = body else { return };
            for id in idents(source, body, b"it") {
                push_at(cop, source, id, AVOID_IT, false, 2, diagnostics);
            }
        }
        _ => {}
    }
}

fn report_num(
    cop: &ItBlockParameter,
    source: &SourceFile,
    body: Node<'_>,
    style: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if style == "disallow" {
        return;
    }
    for id in num_idents(source, body) {
        push_at(cop, source, id, USE_IT, true, 2, diagnostics);
        push_replace(
            corrections,
            id.start_byte(),
            id.end_byte(),
            "it",
            cop.name(),
        );
    }
}

fn report_named(
    cop: &ItBlockParameter,
    source: &SourceFile,
    block: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(name) = plain_arg(source, block) else {
        return;
    };
    let Some(body) = block_body(block) else {
        return;
    };
    let uses = idents(source, body, name);
    if uses.is_empty() {
        return;
    }
    if let Some(params) = block.child_by_field_name("parameters") {
        let (start, end) = param_span(source, params);
        push_replace(corrections, start, end, "", cop.name());
    }
    for id in uses {
        push_at(
            cop,
            source,
            id,
            USE_IT,
            true,
            node_bytes(source, id).len(),
            diagnostics,
        );
        push_replace(
            corrections,
            id.start_byte(),
            id.end_byte(),
            "it",
            cop.name(),
        );
    }
}

impl Cop for ItBlockParameter {
    fn name(&self) -> &'static str {
        "Style/ItBlockParameter"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["block", "do_block"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if !ruby34(config) {
            return;
        }
        let style = config.get_str("EnforcedStyle", "allow_single_line");
        if explicit_params(node) {
            if style == "always" {
                report_named(self, source, node, diagnostics, &mut corrections);
            }
            return;
        }
        if is_it_block(source, node) {
            report_it(self, source, node, style, diagnostics);
            return;
        }
        if let Some(body) = block_body(node) {
            report_num(self, source, body, style, diagnostics, &mut corrections);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ItBlockParameter, "cops/style/it_block_parameter");

    fn ruby_config(ver: &str, style: &str) -> CopConfig {
        let mut config = CopConfig::default();
        let ver: serde_yml::Value = serde_yml::from_str(ver).expect("version");
        config.options.insert("TargetRubyVersion".into(), ver);
        config.options.insert(
            "EnforcedStyle".into(),
            serde_yml::Value::String(style.into()),
        );
        config
    }

    #[test]
    fn skips_before_ruby34() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &ItBlockParameter,
            b"block { do_something(_1) }\n",
            ruby_config("3.3", "always"),
        );
    }

    fn corrected(src: &str) -> String {
        let config = ruby_config("3.4", "always");
        let source = crate::parse::source::SourceFile::from_bytes("t.rb", src.as_bytes().to_vec());
        let tree = crate::parse::parse_ruby(&source).expect("parse");
        let mut corrections = Vec::new();
        walk(
            &ItBlockParameter,
            &source,
            tree.root_node(),
            &config,
            &mut Vec::new(),
            &mut corrections,
        );
        String::from_utf8(
            crate::correction::CorrectionSet::from_vec(corrections).apply(source.as_bytes()),
        )
        .expect("utf8")
    }

    fn walk(
        cop: &ItBlockParameter,
        source: &crate::parse::source::SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<crate::diagnostic::Diagnostic>,
        corrections: &mut Vec<crate::correction::Correction>,
    ) {
        if matches!(node.kind(), "block" | "do_block") {
            cop.check_node(source, node, config, diagnostics, Some(corrections));
        }
        for child in node.children(&mut node.walk()) {
            walk(cop, source, child, config, diagnostics, corrections);
        }
    }

    #[test]
    fn always_drops_space_before_newline() {
        assert_eq!(
            corrected("list.each do |item|\n  item\nend\n"),
            "list.each do\n  it\nend\n"
        );
        assert_eq!(
            corrected("list.each { |item|\n  item\n}\n"),
            "list.each {\n  it\n}\n"
        );
        assert_eq!(
            corrected("list.each { |item| item }\n"),
            "list.each { it }\n"
        );
        assert_eq!(
            corrected("list.each do |item| \t\n  item\nend\n"),
            "list.each do\n  it\nend\n"
        );
        assert_eq!(
            corrected("list.each do |item|\r\n  item\r\nend\r\n"),
            "list.each do\r\n  it\r\nend\r\n"
        );
    }
}
