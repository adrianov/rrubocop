//! Layout/MultilineMethodCallIndentation.

use tree_sitter::{Node, Tree};

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct MultilineMethodCallIndentation;

fn has_dot(source: &SourceFile, node: Node<'_>) -> bool {
    if let Some(op) = node.child_by_field_name("operator") {
        let t = shared::node_bytes(source, op);
        return t == b"." || t == b"&.";
    }
    // Inlined `_call_operator` may appear as an anonymous `.` child.
    let mut cur = node.walk();
    node.children(&mut cur).any(|c| {
        !c.is_named() && matches!(shared::node_bytes(source, c), b"." | b"&.")
    })
}

/// RuboCop `left_hand_side`: walk up dotted call parents so every link in a
/// chain indents from the root expression's line.
///
/// Tree-sitter often nests `expect{}.to change.by.and` with `.and` inside
/// `.to`'s argument list; RuboCop's parser attaches `.and` to the outer
/// expectation. Lift through `argument_list` so indented style matches.
fn chain_root<'a>(source: &SourceFile, node: Node<'a>) -> Node<'a> {
    let mut n = node.child_by_field_name("receiver").unwrap_or(node);
    loop {
        let before = n.start_byte();
        n = walk_dotted_parents(source, n);
        if !lift_from_arg_list(source, &mut n) || n.start_byte() == before {
            break;
        }
    }
    n
}

fn walk_dotted_parents<'a>(source: &SourceFile, mut n: Node<'a>) -> Node<'a> {
    while let Some(parent) = n.parent() {
        if parent.kind() == "call" && has_dot(source, parent) {
            n = parent;
        } else {
            break;
        }
    }
    n
}

fn lift_from_arg_list<'a>(source: &SourceFile, n: &mut Node<'a>) -> bool {
    let Some(args) = n.parent().filter(|p| p.kind() == "argument_list") else {
        return false;
    };
    let Some(call) = args.parent() else {
        return false;
    };
    if call.kind() == "call" && has_dot(source, call) {
        *n = call;
        true
    } else {
        false
    }
}

fn expected_col(source: &SourceFile, node: Node<'_>, recv: Node<'_>, style: &str, width: usize) -> usize {
    match style {
        "indented_relative_to_receiver" => shared::line_indent(source, recv.start_byte()) + width,
        "indented" => shared::line_indent(source, chain_root(source, node).start_byte()) + width,
        _ => shared::node_col(source, recv),
    }
}

fn inside_paren_arg_list(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(n) = p {
        if n.kind() == "argument_list" {
            return n.child(0).is_some_and(|c| !c.is_named() && c.kind() == "(");
        }
        if matches!(n.kind(), "program" | "method" | "singleton_method" | "class" | "module") {
            break;
        }
        p = n.parent();
    }
    false
}

fn check_call(
    cop: &dyn Cop,
    source: &SourceFile,
    n: Node<'_>,
    style: &str,
    width: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some((method, expected, actual)) = call_indent_mismatch(source, n, style, width) else {
        return;
    };
    report::fix_indent(
        cop,
        source,
        method.start_byte(),
        format!(
            "Align method call receivers and their chained calls consistently (expected column {expected})."
        ),
        diagnostics,
        corrections,
        actual,
        expected,
    );
}

fn call_indent_mismatch<'a>(
    source: &SourceFile,
    n: Node<'a>,
    style: &str,
    width: usize,
) -> Option<(Node<'a>, usize, usize)> {
    let (recv, method) = leading_dot_parts(source, n)?;
    let expected = expected_col(source, n, recv, style, width);
    let actual = shared::line_indent(source, method.start_byte());
    (actual != expected).then_some((method, expected, actual))
}

fn leading_dot_parts<'a>(source: &SourceFile, n: Node<'a>) -> Option<(Node<'a>, Node<'a>)> {
    if n.kind() != "call" || !has_dot(source, n) || inside_paren_arg_list(n) {
        return None;
    }
    let recv = n.child_by_field_name("receiver")?;
    let method = n.child_by_field_name("method")?;
    let (recv_end_line, _) = source.offset_to_line_col(recv.end_byte().saturating_sub(1));
    if recv_end_line == shared::node_line(source, method) {
        return None;
    }
    Some((recv, method))
}

impl Cop for MultilineMethodCallIndentation {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodCallIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = code_map;
        let width = config.get_usize("IndentationWidth", 2);
        let style = config.get_str("EnforcedStyle", "aligned");
        shared::for_each_descendant(tree.root_node(), |n| {
            check_call(self, source, n, style, width, diagnostics, &mut corrections);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;
    crate::cop_fixture_tests!(MultilineMethodCallIndentation, "cops/layout/multiline_method_call_indentation");

    #[test]
    fn indented_rspec_chains() {
        let mut config = CopConfig::default();
        config.options.insert("EnforcedStyle".into(), serde_yml::Value::String("indented".into()));
        config.options.insert("IndentationWidth".into(), serde_yml::Value::Number(2.into()));
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &MultilineMethodCallIndentation,
            b"expect { post(:create) }\n  .to change(A, :count).by(1)\n  .and(change(B, :count).by(1))\n",
            config,
        );
    }
}
