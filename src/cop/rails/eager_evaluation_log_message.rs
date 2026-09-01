//! Rails/EagerEvaluationLogMessage — interpolated strings passed to `Rails.logger.debug`.

use tree_sitter::Node;

use crate::cop::shared::{
    argument_nodes, call_method_name, call_receiver, for_each_descendant, is_const_named,
    method_node, push_replace,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EagerEvaluationLogMessage;

fn is_rails_logger(source: &SourceFile, node: Node<'_>) -> bool {
    call_method_name(source, node) == Some(b"logger")
        && call_receiver(node).is_some_and(|r| is_const_named(source, r, b"Rails"))
}

fn is_rails_logger_debug(source: &SourceFile, node: Node<'_>) -> bool {
    call_method_name(source, node) == Some(b"debug")
        && call_receiver(node)
            .is_some_and(|r| r.kind() == "call" && is_rails_logger(source, r))
}

fn has_interpolation(node: Node<'_>) -> bool {
    let mut found = false;
    for_each_descendant(node, |n| {
        if n.kind() == "interpolation" {
            found = true;
        }
    });
    found
}

fn interpolated_arg(node: Node<'_>) -> Option<Node<'_>> {
    let arg = argument_nodes(node).into_iter().next()?;
    if !matches!(
        arg.kind(),
        "string" | "chained_string" | "heredoc_body" | "heredoc_beginning"
    ) {
        return None;
    }
    has_interpolation(arg).then_some(arg)
}

fn parenthesized_call(bytes: &[u8], after_method: usize) -> bool {
    let mut p = after_method;
    while p < bytes.len() && bytes[p].is_ascii_whitespace() {
        p += 1;
    }
    p < bytes.len() && bytes[p] == b'('
}

fn block_text(arg_src: &str, parenthesized: bool) -> String {
    if parenthesized {
        format!(" {{ {arg_src} }}")
    } else {
        format!("{{ {arg_src} }}")
    }
}

fn replacement(source: &SourceFile, node: Node<'_>, arg: Node<'_>) -> Option<(usize, usize, String)> {
    let meth = method_node(node)?;
    let start = meth.end_byte();
    let end = node.end_byte();
    let parenthesized = parenthesized_call(source.as_bytes(), start);
    let range_start = if parenthesized { start } else { (start + 1).min(end) };
    let arg_src = String::from_utf8_lossy(&source.as_bytes()[arg.start_byte()..arg.end_byte()]);
    Some((range_start, end, block_text(&arg_src, parenthesized)))
}

impl Cop for EagerEvaluationLogMessage {
    fn name(&self) -> &'static str {
        "Rails/EagerEvaluationLogMessage"
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
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"debug"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if node.child_by_field_name("block").is_some() {
            return;
        }
        if !is_rails_logger_debug(source, node) {
            return;
        }
        let Some(arg) = interpolated_arg(node) else {
            return;
        };
        let Some((start, end, text)) = replacement(source, node, arg) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(arg.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            "Pass a block to `Rails.logger.debug`.".to_string(),
        );
        if push_replace(&mut corrections, start, end, text, self.name()) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correction::CorrectionSet;

    crate::cop_fixture_tests!(EagerEvaluationLogMessage, "cops/rails/eager_evaluation_log_message");

    fn fixed(src: &[u8]) -> String {
        let source = SourceFile::from_bytes("test.rb", src.to_vec());
        let tree = crate::parse::parse_ruby(&source).expect("parse");
        let node = tree.root_node().named_child(0).expect("call");
        let cop = EagerEvaluationLogMessage;
        let mut corrections = Vec::new();
        cop.check_node(
            &source,
            node,
            &CopConfig::default(),
            &mut Vec::new(),
            Some(&mut corrections),
        );
        String::from_utf8_lossy(&CorrectionSet::from_vec(corrections).apply(src)).into_owned()
    }

    #[test]
    fn autocorrect_parenthesized_call() {
        assert_eq!(
            fixed(b"Rails.logger.debug(\"msg #{x}\")"),
            "Rails.logger.debug { \"msg #{x}\" }"
        );
    }

    #[test]
    fn autocorrect_unparenthesized_call() {
        assert_eq!(
            fixed(b"Rails.logger.debug \"msg #{x}\""),
            "Rails.logger.debug { \"msg #{x}\" }"
        );
    }
}
