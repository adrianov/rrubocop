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

/// RuboCop aligns `rescue`/`ensure` with `def` / `begin` / `do` / `class` / `module`.
fn alignment_col(source: &SourceFile, node: Node<'_>) -> Option<usize> {
    let mut p = node.parent();
    while let Some(n) = p {
        if let Some(kw) = align_kw(n.kind()) {
            return Some(kw_col(source, n, kw));
        }
        p = n.parent();
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
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if node.parent().is_some_and(|p| p.kind() == node.kind()) {
            return;
        }
        let Some(base_col) = alignment_col(source, node) else {
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
