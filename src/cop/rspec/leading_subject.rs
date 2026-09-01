//! RSpec/LeadingSubject — `subject` must come before `let` helpers.

use tree_sitter::Node;

use crate::cop::rspec::helpers::{
    bare_rspec_call, block_body, call_block, is_group, is_let_helper, RSPEC_INCLUDE,
};
use crate::cop::shared::{call_method_name, method_node};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct LeadingSubject;

const MSG: &str = "Declare `subject` above any other `let` declarations.";

fn is_subject(method: &[u8]) -> bool {
    method == b"subject" || method == b"subject!"
}

fn group_stmts(body: Node<'_>) -> Vec<Node<'_>> {
    if matches!(body.kind(), "body_statement" | "statements") {
        let mut cur = body.walk();
        body.named_children(&mut cur).collect()
    } else {
        vec![body]
    }
}

fn report_subject(cop: &LeadingSubject, source: &SourceFile, call: Node<'_>, diagnostics: &mut Vec<Diagnostic>) {
    let meth = method_node(call).unwrap_or(call);
    let (line, col) = source.offset_to_line_col(meth.start_byte());
    diagnostics.push(cop.diagnostic(source, line, col, MSG.into()));
}

fn check_group_body(
    source: &SourceFile,
    body: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    cop: &LeadingSubject,
) {
    let mut seen_let = false;
    for stmt in group_stmts(body) {
        if !matches!(stmt.kind(), "call" | "command") {
            continue;
        }
        let Some(method) = call_method_name(source, stmt) else {
            continue;
        };
        if is_subject(method) {
            if seen_let {
                report_subject(cop, source, stmt, diagnostics);
            }
        } else if is_let_helper(method) {
            seen_let = true;
        }
    }
}

impl Cop for LeadingSubject {
    fn name(&self) -> &'static str {
        "RSpec/LeadingSubject"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_INCLUDE
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
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(method) = bare_rspec_call(source, node) else {
            return;
        };
        if !is_group(method) {
            return;
        }
        let Some(body) = call_block(node).and_then(block_body) else {
            return;
        };
        check_group_body(source, body, diagnostics, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(LeadingSubject, "cops/rspec/leading_subject");
}
