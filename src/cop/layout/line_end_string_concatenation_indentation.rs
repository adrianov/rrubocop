//! Layout/LineEndStringConcatenationIndentation.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct LineEndStringConcatenationIndentation;

fn concat_msg(align: bool) -> String {
    if align {
        "Align parts of a string concatenated with backslash.".into()
    } else {
        "Indent the first part of a string concatenated with backslash.".into()
    }
}

/// RuboCop `PARENT_TYPES_FOR_INDENTED` — direct parent of the chained string.
fn always_indented(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return true;
    };
    matches!(
        parent.kind(),
        "method"
            | "singleton_method"
            | "do_block"
            | "block"
            | "lambda"
            | "if"
            | "unless"
            | "then"
            | "else"
            | "elsif"
            | "begin"
            | "program"
            | "body_statement"
    )
}

fn string_parts<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    shared::named_kids(node)
        .into_iter()
        .filter(|n| matches!(n.kind(), "string" | "heredoc_beginning" | "chained_string"))
        .collect()
}

fn single_line(source: &SourceFile, n: Node<'_>) -> bool {
    let (s, _) = source.offset_to_line_col(n.start_byte());
    let (e, _) = source.offset_to_line_col(n.end_byte().saturating_sub(1).max(n.start_byte()));
    s == e
}

fn backslash_between(source: &SourceFile, left: Node<'_>, right: Node<'_>) -> bool {
    let bytes = source.as_bytes();
    let start = left.end_byte();
    let end = right.start_byte();
    end > start && bytes[start..end].contains(&b'\\')
}

fn pair_on_new_line(source: &SourceFile, left: Node<'_>, right: Node<'_>) -> bool {
    shared::node_line(source, left) != shared::node_line(source, right)
}

fn multiline_backslash_parts(source: &SourceFile, parts: &[Node<'_>]) -> bool {
    if parts.len() < 2 || parts.iter().any(|p| !single_line(source, *p)) {
        return false;
    }
    if !pair_on_new_line(source, parts[0], parts[parts.len() - 1]) {
        return false;
    }
    parts.windows(2).all(|pair| {
        !pair_on_new_line(source, pair[0], pair[1]) || backslash_between(source, pair[0], pair[1])
    })
}

fn base_column(source: &SourceFile, first: Node<'_>) -> usize {
    if let Some(parent) = first.parent() {
        if let Some(gp) = parent.parent() {
            if gp.kind() == "pair" {
                return shared::node_col(source, gp);
            }
        }
    }
    shared::line_indent(source, first.start_byte())
}

fn report_indent(
    cop: &dyn Cop,
    source: &SourceFile,
    part: Node<'_>,
    expected: usize,
    align_msg: bool,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let actual = shared::node_col(source, part);
    if actual == expected {
        return;
    }
    let ls = source
        .line_start(shared::node_line(source, part))
        .unwrap_or(part.start_byte());
    report::report_fix(
        cop,
        source,
        part.start_byte(),
        concat_msg(align_msg),
        diagnostics,
        corrections,
        ls,
        ls + actual,
        " ".repeat(expected),
    );
}

fn align_tail(
    cop: &dyn Cop,
    source: &SourceFile,
    parts: &[Node<'_>],
    columns: &[usize],
    from: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut base = columns[from];
    for (i, &col) in columns[from + 1..].iter().enumerate() {
        if col != base {
            report_indent(
                cop,
                source,
                parts[from + 1 + i],
                base,
                true,
                diagnostics,
                corrections,
            );
        }
        base = col;
    }
}

fn check_parts(
    cop: &dyn Cop,
    source: &SourceFile,
    parts: &[Node<'_>],
    use_indented: bool,
    width: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let columns: Vec<usize> = parts.iter().map(|p| shared::node_col(source, *p)).collect();
    if use_indented {
        report_indent(
            cop,
            source,
            parts[1],
            base_column(source, parts[0]) + width,
            false,
            diagnostics,
            corrections,
        );
        align_tail(cop, source, parts, &columns, 1, diagnostics, corrections);
    } else {
        align_tail(cop, source, parts, &columns, 0, diagnostics, corrections);
    }
}

impl Cop for LineEndStringConcatenationIndentation {
    fn name(&self) -> &'static str {
        "Layout/LineEndStringConcatenationIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["chained_string"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let parts = string_parts(node);
        if !multiline_backslash_parts(source, &parts) {
            return;
        }
        let use_indented =
            config.get_str("EnforcedStyle", "aligned") == "indented" || always_indented(node);
        check_parts(
            self,
            source,
            &parts,
            use_indented,
            config.get_usize("IndentationWidth", 2),
            diagnostics,
            &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;

    crate::cop_fixture_tests!(
        LineEndStringConcatenationIndentation,
        "cops/layout/line_end_string_concatenation_indentation"
    );

    #[test]
    fn indented_style_allows_aligned_tail_parts() {
        let mut config = CopConfig::default();
        config.options.insert(
            "EnforcedStyle".into(),
            serde_yml::Value::String("indented".into()),
        );
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &LineEndStringConcatenationIndentation,
            b"def t\n  \"a\" \\\n    \"b\" \\\n    \"c\".html_safe\nend\n",
            config,
        );
    }
}
