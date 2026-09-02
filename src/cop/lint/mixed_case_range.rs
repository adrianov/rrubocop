use tree_sitter::Node;

use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/MixedCaseRange — 'A'..'z' style ranges.
pub struct MixedCaseRange;

fn char_content(source: &SourceFile, node: Node<'_>) -> Option<char> {
    let t = node_text(source, node);
    let inner = t.trim_matches(|c| c == '\'' || c == '"');
    let mut chars = inner.chars();
    let c = chars.next()?;
    chars.next().is_none().then_some(c)
}

fn mixed_case(a: char, b: char) -> bool {
    (a.is_ascii_uppercase() && b.is_ascii_lowercase())
        || (a.is_ascii_lowercase() && b.is_ascii_uppercase())
}

fn range_chars(source: &SourceFile, node: Node<'_>) -> Option<(char, char)> {
    let begin = node.child_by_field_name("begin")?;
    let end = node.child_by_field_name("end")?;
    if begin.kind() != "string" || end.kind() != "string" {
        return None;
    }
    Some((char_content(source, begin)?, char_content(source, end)?))
}

impl Cop for MixedCaseRange {
    fn name(&self) -> &'static str {
        "Lint/MixedCaseRange"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["range"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some((a, b)) = range_chars(source, node) else {
            return;
        };
        if !mixed_case(a, b) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Ranges from upper to lower case ASCII letters may include unintended characters. Prefer explicit ranges like ('A'..'Z') and ('a'..'z').".to_string(),
        ));
    }
}
