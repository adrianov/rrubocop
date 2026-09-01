//! Layout/ElseAlignment.

use tree_sitter::Node;

use crate::cop::layout::end_align::{assignment_context_base_col, same_line_assign_col};
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

fn kw_or_node_col(source: &SourceFile, n: Node<'_>, kw: &str) -> Option<usize> {
    keyword_col(source, n, kw).or_else(|| Some(shared::node_col(source, n)))
}

fn last_when_col(source: &SourceFile, case_node: Node<'_>) -> Option<usize> {
    let mut cur = case_node.walk();
    let last = case_node
        .named_children(&mut cur)
        .filter(|n| matches!(n.kind(), "when" | "in"))
        .last()?;
    keyword_col(source, last, last.kind()).or_else(|| Some(shared::node_col(source, last)))
}

fn rescue_label(from: Node<'_>) -> &'static str {
    let mut p = from.parent();
    while let Some(n) = p {
        match n.kind() {
            "begin" => return "begin",
            "method" | "singleton_method" => return "def",
            "do_block" | "block" => return "do",
            _ => p = n.parent(),
        }
    }
    "rescue"
}

fn do_block_align_col(source: &SourceFile, n: Node<'_>) -> Option<usize> {
    // When `do` trails a call (`open(...) do`), align rescue/else/end with the
    // line indent (call start), not the `do` keyword column — matches RuboCop.
    Some(shared::line_indent(source, n.start_byte()))
}

fn ancestor_kw_col(source: &SourceFile, from: Node<'_>) -> Option<usize> {
    let mut p = Some(from);
    while let Some(n) = p {
        let col = match n.kind() {
            "begin" => kw_or_node_col(source, n, "begin"),
            "method" | "singleton_method" => kw_or_node_col(source, n, "def"),
            "do_block" | "block" => do_block_align_col(source, n),
            _ => None,
        };
        if col.is_some() {
            return col;
        }
        p = n.parent();
    }
    Some(shared::node_col(source, from))
}

fn alignment_label(else_node: Node<'_>) -> &'static str {
    let Some(parent) = else_node.parent() else {
        return "keyword";
    };
    match parent.kind() {
        "case" => "when",
        "case_match" => "in",
        "if" | "unless" | "elsif" => parent.kind(),
        "begin" => "begin",
        "method" | "singleton_method" => "def",
        "rescue" | "body_statement" => rescue_label(parent),
        other => other,
    }
}

fn if_kw_offset(parent: Node<'_>, kw: &str) -> usize {
    parent
        .children(&mut parent.walk())
        .find(|c| !c.is_named() && c.kind() == kw)
        .map(|c| c.start_byte())
        .unwrap_or(parent.start_byte())
}

fn if_align_col(source: &SourceFile, parent: Node<'_>, end_style: &str) -> Option<usize> {
    let kw = parent.kind();
    let kw_c = kw_or_node_col(source, parent, kw)?;
    if !matches!(end_style, "variable" | "start_of_line") {
        return Some(kw_c);
    }
    Some(variable_if_align_col(source, parent, kw).unwrap_or(kw_c))
}

/// RuboCop CheckAssignment: same-line `||=` / `=` / mass-assign → LHS column.
/// Walk from the outermost `if` when parent is `elsif`.
fn variable_if_align_col(source: &SourceFile, parent: Node<'_>, kw: &str) -> Option<usize> {
    let assign_anchor = outermost_if(parent);
    same_line_assign_col(source, assign_anchor)
        .or_else(|| {
            assignment_context_base_col(source, if_kw_offset(assign_anchor, assign_anchor.kind()))
        })
        .or_else(|| assignment_context_base_col(source, if_kw_offset(parent, kw)))
}

fn outermost_if(mut node: Node<'_>) -> Node<'_> {
    while let Some(p) = node.parent() {
        if matches!(p.kind(), "if" | "unless") {
            node = p;
            continue;
        }
        break;
    }
    node
}

fn base_col_for_else(source: &SourceFile, else_node: Node<'_>, end_style: &str) -> Option<usize> {
    let parent = else_node.parent()?;
    match parent.kind() {
        "case" | "case_match" => last_when_col(source, parent),
        "if" | "unless" | "elsif" => if_align_col(source, parent, end_style),
        "begin" => kw_or_node_col(source, parent, "begin"),
        "method" | "singleton_method" => kw_or_node_col(source, parent, "def"),
        "do_block" | "block" => do_block_align_col(source, parent),
        "rescue" | "body_statement" => ancestor_kw_col(source, parent),
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
        if shared::node_col(source, node) != shared::line_indent(source, node.start_byte()) {
            return;
        }
        let end_style = config.get_str("EndAlignmentStyle", "keyword");
        let Some(base_col) = base_col_for_else(source, node, end_style) else {
            return;
        };
        if shared::node_col(source, node) == base_col {
            return;
        }
        let base_kw = alignment_label(node);
        let kw = node.kind();
        report::fix_indent(
            self,
            source,
            node.start_byte(),
            format!("Align `{kw}` with `{base_kw}`."),
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
    use crate::cop::CopConfig;
    use crate::testutil::assert_cop_no_offenses_full_with_config;

    crate::cop_fixture_tests!(ElseAlignment, "cops/layout/else_alignment");

    #[test]
    fn variable_style_or_asgn_if_else_no_offense() {
        let mut config = CopConfig::default();
        config
            .options
            .insert("EndAlignmentStyle".into(), serde_yml::Value::String("variable".into()));
        assert_cop_no_offenses_full_with_config(
            &ElseAlignment,
            b"      @x ||= if cond\n        1\n      else\n        2\n      end\n",
            config,
        );
    }

    #[test]
    fn variable_style_mass_assign_if_else_no_offense() {
        let mut config = CopConfig::default();
        config
            .options
            .insert("EndAlignmentStyle".into(), serde_yml::Value::String("variable".into()));
        assert_cop_no_offenses_full_with_config(
            &ElseAlignment,
            b"          a, b = if cond\n            [1, 2]\n          else\n            [3, 4]\n          end\n",
            config,
        );
    }
}
