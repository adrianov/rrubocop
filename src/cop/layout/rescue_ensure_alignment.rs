//! Layout/RescueEnsureAlignment.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RescueEnsureAlignment;

fn kw_col(source: &SourceFile, n: Node<'_>, kw: &str) -> usize {
    let mut cur = n.walk();
    n.children(&mut cur)
        .find(|c| !c.is_named() && c.kind() == kw)
        .map(|c| shared::node_col(source, c))
        .unwrap_or_else(|| shared::node_col(source, n))
}

fn align_kw(kind: &str) -> Option<&'static str> {
    match kind {
        "method" | "singleton_method" => Some("def"),
        "begin" => Some("begin"),
        "do_block" | "block" => Some("do"),
        "class" => Some("class"),
        "module" => Some("module"),
        _ => None,
    }
}

fn ancestor_align_node(node: Node<'_>) -> Option<Node<'_>> {
    let mut p = node.parent();
    while let Some(n) = p {
        if align_kw(n.kind()).is_some() {
            return Some(n);
        }
        p = n.parent();
    }
    None
}

/// When Layout/BeginEndAlignment uses `start_of_line`, align to the first
/// non-whitespace on the ancestor's starting line (RuboCop parity).
fn alignment_col(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> Option<usize> {
    let ancestor = ancestor_align_node(node)?;
    if matches!(ancestor.kind(), "do_block" | "block") {
        return block_expression_col(source, ancestor);
    }
    if begin_end_start_of_line(config) {
        return Some(shared::line_indent(source, ancestor.start_byte()));
    }
    Some(kw_col(source, ancestor, align_kw(ancestor.kind())?))
}

/// Parser `block` nodes start at the send; tree-sitter `do_block` starts at `do`.
fn block_expression_col(source: &SourceFile, ancestor: Node<'_>) -> Option<usize> {
    let send = block_send_node(ancestor)?;
    Some(shared::line_indent(source, outermost_send(send).start_byte()))
}

fn outermost_send(mut send: Node<'_>) -> Node<'_> {
    while let Some(parent) = send.parent() {
        if matches!(parent.kind(), "call" | "command") {
            send = parent;
        } else {
            break;
        }
    }
    send
}

fn begin_end_start_of_line(config: &CopConfig) -> bool {
    config.get_bool("BeginEndAlignmentEnabled", true)
        && config.get_str("BeginEndAlignmentStyle", "begin") == "start_of_line"
}

fn block_send_node(block: Node<'_>) -> Option<Node<'_>> {
    // `(call/command) do ... end` — send is previous named sibling or parent call.
    if let Some(prev) = block.prev_named_sibling() {
        if matches!(prev.kind(), "call" | "command" | "element_reference") {
            return Some(prev);
        }
    }
    let parent = block.parent()?;
    if matches!(parent.kind(), "call" | "command") {
        return Some(parent);
    }
    None
}

impl Cop for RescueEnsureAlignment {
    fn name(&self) -> &'static str {
        "Layout/RescueEnsureAlignment"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["rescue", "ensure"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if node.parent().is_some_and(|p| p.kind() == node.kind()) {
            return;
        }
        let Some(base_col) = alignment_col(source, node, config) else {
            return;
        };
        if shared::node_col(source, node) == base_col {
            return;
        }
        let (kl, kc) = source.offset_to_line_col(node.start_byte());
        report::fix_indent(
            self,
            source,
            node.start_byte(),
            format!(
                "`{}` at {kl}, {kc} is not aligned with beginning at column {base_col}.",
                node.kind()
            ),
            diagnostics,
            &mut corrections,
            shared::line_indent(source, node.start_byte()),
            base_col,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RescueEnsureAlignment, "cops/layout/rescue_ensure_alignment");
}
