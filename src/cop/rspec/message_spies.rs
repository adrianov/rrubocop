//! RSpec/MessageSpies — `expect(x).to receive` vs `have_received`.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, method_node, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MessageSpies;

const HAVE_RECEIVED: &str = "have_received";

fn is_spy(name: &[u8]) -> bool {
    matches!(name, b"receive" | b"have_received")
}

fn expect_source(source: &SourceFile, runner: Node<'_>) -> Option<String> {
    let recv = call_receiver(runner)?;
    if call_receiver(recv).is_some() || call_method_name(source, recv) != Some(b"expect") {
        return None;
    }
    argument_nodes(recv)
        .into_iter()
        .next()
        .map(|arg| node_text(source, arg))
}

fn bare_name<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    if !matches!(node.kind(), "call" | "command" | "command_call") {
        return None;
    }
    if call_receiver(node).is_some() {
        return None;
    }
    call_method_name(source, node)
}

fn spy_message(style: &str, expect_src: &str) -> String {
    if style == "receive" {
        "Prefer `receive` for setting message expectations.".to_string()
    } else {
        format!(
            "Prefer `{HAVE_RECEIVED}` for setting message expectations. Setup `{expect_src}` as a spy using `allow` or `instance_spy`."
        )
    }
}

fn push_spy(
    cop: &MessageSpies,
    source: &SourceFile,
    node: Node<'_>,
    message: String,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let meth = method_node(node).unwrap_or(node);
    let (line, col) = source.offset_to_line_col(meth.start_byte());
    let mut diag = cop.diagnostic(source, line, col, message);
    diag.highlight_length = meth.end_byte().saturating_sub(meth.start_byte()).max(1);
    diagnostics.push(diag);
}

/// Walk in source order. A preferred matcher stops the search, as RuboCop does.
fn walk_spies(
    cop: &MessageSpies,
    source: &SourceFile,
    node: Node<'_>,
    style: &str,
    expect_src: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    if let Some(name) = bare_name(source, node).filter(|n| is_spy(n)) {
        if name == style.as_bytes() {
            return true;
        }
        push_spy(
            cop,
            source,
            node,
            spy_message(style, expect_src),
            diagnostics,
        );
    }
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        if walk_spies(cop, source, child, style, expect_src, diagnostics) {
            return true;
        }
    }
    false
}

impl Cop for MessageSpies {
    fn name(&self) -> &'static str {
        "RSpec/MessageSpies"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "command_call"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"to", b"not_to", b"to_not"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        if !matches!(method, b"to" | b"not_to" | b"to_not") {
            return;
        }
        let Some(expect_src) = expect_source(source, node) else {
            return;
        };
        walk_spies(
            self,
            source,
            node,
            &config.get_str("EnforcedStyle", HAVE_RECEIVED),
            &expect_src,
            diagnostics,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(MessageSpies, "cops/rspec/message_spies");

    #[test]
    fn receive_style_flags_have_received() {
        let mut config = CopConfig::default();
        config.options.insert(
            "EnforcedStyle".into(),
            serde_yml::Value::String("receive".into()),
        );
        crate::testutil::assert_cop_offenses_full_with_config(
            &MessageSpies,
            b"expect(foo).to have_received(:bar)\n               ^^^^^^^^^^^^^ RSpec/MessageSpies: Prefer `receive` for setting message expectations.\n",
            config,
        );
    }
}
