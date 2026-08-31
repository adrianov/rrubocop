//! Layout/ElseAlignment.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ElseAlignment;

fn keyword_col(source: &SourceFile, n: Node<'_>, kw: &str) -> Option<usize> {
    let mut cur = n.walk();
    n.children(&mut cur)
        .find(|c| !c.is_named() && c.kind() == kw)
        .map(|c| shared::node_col(source, c))
}

fn last_when_col(source: &SourceFile, case_node: Node<'_>) -> Option<usize> {
    let mut cur = case_node.walk();
    let whens: Vec<_> = case_node
        .named_children(&mut cur)
        .filter(|n| matches!(n.kind(), "when" | "in"))
        .collect();
    let last = *whens.last()?;
    keyword_col(source, last, last.kind()).or_else(|| Some(shared::node_col(source, last)))
}

fn base_col_for_else(source: &SourceFile, else_node: Node<'_>) -> Option<usize> {
    let parent = else_node.parent()?;
    match parent.kind() {
        "case" => last_when_col(source, parent),
        "if" | "unless" | "elsif" => {
            // Align with if/unless keyword (not mid-line assignment start).
            keyword_col(source, parent, parent.kind())
                .or_else(|| Some(shared::node_col(source, parent)))
        }
        "rescue" => {
            // Align with rescue / surrounding def start-of-line — use rescue keyword.
            keyword_col(source, parent, "rescue")
                .or_else(|| Some(shared::node_col(source, parent)))
        }
        _ => Some(shared::node_col(source, parent)),
    }
}

impl Cop for ElseAlignment {
    fn name(&self) -> &'static str {
        "Layout/ElseAlignment"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["else", "elsif"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        // Only check else/elsif that begin their line (RuboCop begins_its_line?).
        if shared::node_col(source, node) != shared::line_indent(source, node.start_byte()) {
            return;
        }
        let Some(base_col) = base_col_for_else(source, node) else {
            return;
        };
        if shared::node_col(source, node) == base_col {
            return;
        }
        let parent_kind = node.parent().map(|p| p.kind()).unwrap_or("keyword");
        let kw = node.kind();
        report::fix_indent(
            self,
            source,
            node.start_byte(),
            format!("Align `{kw}` with `{parent_kind}`."),
            diagnostics,
            &mut corrections,
            shared::line_indent(source, node.start_byte()),
            base_col,
        );
    }
}
