//! RSpec/IteratedExpectation — prefer `all` matcher over `.each` + `expect`.

use tree_sitter::Node;

use crate::cop::rspec::helpers::{block_body, RSPEC_INCLUDE};
use crate::cop::shared::{
    argument_nodes, call_method_name, call_receiver, for_each_descendant, method_node, node_bytes,
    push_replace,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct IteratedExpectation;

const MSG: &str = "Prefer using the `all` matcher instead of iterating over an array.";

fn each_call<'a>(source: &SourceFile, block: Node<'a>) -> Option<Node<'a>> {
    let call = block.parent()?;
    if !matches!(call.kind(), "call" | "command" | "command_call") {
        return None;
    }
    (call_method_name(source, call) == Some(b"each")).then_some(call)
}

/// Single block arg, or `_1` when parameters are omitted (RuboCop numblock).
fn block_arg_name<'a>(source: &'a SourceFile, block: Node<'_>) -> Option<&'a [u8]> {
    let Some(params) = block.child_by_field_name("parameters") else {
        return Some(b"_1");
    };
    let mut cur = params.walk();
    let ids: Vec<_> = params
        .named_children(&mut cur)
        .filter(|n| n.kind() == "identifier")
        .collect();
    (ids.len() == 1).then(|| node_bytes(source, ids[0]))
}

/// RuboCop `expectation?`: `expect(arg).to …` (not `not_to`).
fn is_expectation(source: &SourceFile, node: Node<'_>, arg: &[u8]) -> bool {
    if call_method_name(source, node) != Some(b"to") {
        return false;
    }
    let Some(recv) = call_receiver(node) else {
        return false;
    };
    if call_method_name(source, recv) != Some(b"expect") {
        return false;
    }
    argument_nodes(recv)
        .into_iter()
        .next()
        .is_some_and(|a| a.kind() == "identifier" && node_bytes(source, a) == arg)
}

fn body_nodes(body: Node<'_>) -> Vec<Node<'_>> {
    match body.kind() {
        "body_statement" | "block_body" => {
            let mut cur = body.walk();
            body.named_children(&mut cur)
                .filter(|n| n.kind() != "comment")
                .collect()
        }
        _ => vec![body],
    }
}

/// True if `n` is the method name of a call (not a local read).
fn is_call_method(n: Node<'_>) -> bool {
    n.parent().is_some_and(|p| {
        matches!(p.kind(), "call" | "command" | "command_call")
            && method_node(p).is_some_and(|m| m.id() == n.id())
    })
}

/// Block-arg local used inside the matcher (RuboCop `lvar`); method names do not count.
fn uses_arg_in_matcher(source: &SourceFile, matcher: Node<'_>, arg: &[u8]) -> bool {
    let mut found = false;
    for_each_descendant(matcher, |n| {
        if n.kind() != "identifier" || node_bytes(source, n) != arg {
            return;
        }
        // Bare `to be_valid` is an identifier (method), not a local.
        if n.id() == matcher.id() || is_call_method(n) {
            return;
        }
        found = true;
    });
    found
}

/// Single `to` with one matcher arg that does not reference the block arg.
fn autocorrect_text(
    source: &SourceFile,
    call: Node<'_>,
    to_call: Node<'_>,
    arg: &[u8],
) -> Option<String> {
    let args = argument_nodes(to_call);
    if args.len() != 1 || uses_arg_in_matcher(source, args[0], arg) {
        return None;
    }
    let recv = call_receiver(call)?;
    let collection = std::str::from_utf8(node_bytes(source, recv)).ok()?;
    let matcher = std::str::from_utf8(node_bytes(source, args[0])).ok()?;
    Some(format!("expect({collection}).to all({matcher})"))
}

fn report(
    cop: &IteratedExpectation,
    source: &SourceFile,
    call: Node<'_>,
    replacement: Option<String>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(call.start_byte());
    let mut diag = cop.diagnostic(source, line, col, MSG.into());
    match replacement {
        Some(text) => {
            if push_replace(
                corrections,
                call.start_byte(),
                call.end_byte(),
                text,
                cop.name(),
            ) {
                diag.corrected = true;
            }
        }
        None => diag.correctable = false,
    }
    diagnostics.push(diag);
}

impl Cop for IteratedExpectation {
    fn name(&self) -> &'static str {
        "RSpec/IteratedExpectation"
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_INCLUDE
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["do_block", "block"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(call) = each_call(source, node) else {
            return;
        };
        let Some(arg) = block_arg_name(source, node) else {
            return;
        };
        let Some(body) = block_body(node) else {
            return;
        };
        let stmts = body_nodes(body);
        if stmts.is_empty() || !stmts.iter().all(|n| is_expectation(source, *n, arg)) {
            return;
        }
        report(
            self,
            source,
            call,
            (stmts.len() == 1)
                .then(|| autocorrect_text(source, call, stmts[0], arg))
                .flatten(),
            diagnostics,
            &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::run_cop_full;

    crate::cop_fixture_tests!(IteratedExpectation, "cops/rspec/iterated_expectation");

    fn walk_corr(src: &[u8]) -> (Vec<Diagnostic>, Vec<Correction>) {
        let source = SourceFile::from_bytes("t_spec.rb", src.to_vec());
        let tree = crate::parse::parse_ruby(&source).unwrap();
        let mut corr = Vec::new();
        let mut out = Vec::new();
        let cfg = CopConfig::default();
        crate::cop::walker::BatchedWalker::new(vec![&IteratedExpectation], vec![&cfg]).walk(
            &source,
            tree.root_node(),
            &mut out,
            Some(&mut corr),
        );
        (out, corr)
    }

    #[test]
    fn autocorrects_single_expectation() {
        let src = b"it 'x' do\n  [a, b].each { |u| expect(u).to be_valid }\nend\n";
        assert!(run_cop_full(&IteratedExpectation, src)[0].correctable);
        let (_, corr) = walk_corr(src);
        assert_eq!(corr.len(), 1);
        let mut bytes = src.to_vec();
        bytes.splice(corr[0].start..corr[0].end, corr[0].replacement.bytes());
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap(),
            "it 'x' do\n  expect([a, b]).to all(be_valid)\nend\n"
        );
    }

    #[test]
    fn no_autocorrect_when_matcher_uses_block_arg() {
        let (out, corr) = walk_corr(b"it 'x' do\n  [a].each { |u| expect(u).to eq(u) }\nend\n");
        assert_eq!(out.len(), 1);
        assert!(corr.is_empty());
        assert!(!out[0].correctable);
    }

    #[test]
    fn autocorrects_when_matcher_name_matches_block_arg() {
        let (_, corr) =
            walk_corr(b"it 'x' do\n  [a].each { |be_valid| expect(be_valid).to be_valid }\nend\n");
        assert_eq!(corr.len(), 1);
        assert_eq!(corr[0].replacement, "expect([a]).to all(be_valid)");
    }

    #[test]
    fn multi_expect_not_marked_correctable() {
        let diags = run_cop_full(
            &IteratedExpectation,
            b"it 'x' do\n  [a].each { |u|\n    expect(u).to be_valid\n    expect(u).to be_ok\n  }\nend\n",
        );
        assert_eq!(diags.len(), 1);
        assert!(!diags[0].correctable);
    }
}
