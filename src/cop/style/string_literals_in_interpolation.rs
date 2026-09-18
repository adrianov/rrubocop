//! Style/StringLiteralsInInterpolation.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::style::heuristics::double_quotes_required;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct StringLiteralsInInterpolation;

impl Cop for StringLiteralsInInterpolation {
    fn name(&self) -> &'static str {
        "Style/StringLiteralsInInterpolation"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["interpolation"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "single_quotes");
        let mut cur = node.walk();
        for child in node.named_children(&mut cur) {
            report_string(self, source, child, style, diagnostics);
        }
    }
}

fn string_style_bad(b: &[u8], style: &str) -> bool {
    match style {
        "single_quotes" => b.starts_with(b"\"") && !double_quotes_required(b),
        "double_quotes" => b.starts_with(b"'"),
        _ => false,
    }
}

fn nested_interpolation(child: Node<'_>) -> bool {
    let mut cur = child.walk();
    child
        .named_children(&mut cur)
        .any(|c| c.kind() == "interpolation")
}

fn report_string(
    cop: &StringLiteralsInInterpolation,
    source: &SourceFile,
    child: Node<'_>,
    style: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if child.kind() != "string" || nested_interpolation(child) {
        return;
    }
    if !string_style_bad(node_bytes(source, child), style) {
        return;
    }
    let (line, col) = source.offset_to_line_col(child.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!("Prefer {style} for strings inside interpolations."),
    ));
}
