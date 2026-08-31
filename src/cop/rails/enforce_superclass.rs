//! Shared Application* superclass enforcement (Rails).

use tree_sitter::Node;

use crate::cop::shared::{node_text, push_replace};
use crate::cop::Cop;
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

fn matches_base(text: &str, base_path: &str) -> bool {
    let trimmed = text.trim().trim_start_matches("::");
    trimmed == base_path || trimmed.ends_with(&format!("::{base_path}"))
}

fn report_superclass(
    cop: &dyn Cop,
    source: &SourceFile,
    superclass: Node<'_>,
    prefer: &str,
    msg: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(superclass.start_byte());
    let mut diag = cop.diagnostic(source, line, col, msg.to_string());
    if push_replace(
        corrections,
        superclass.start_byte(),
        superclass.end_byte(),
        prefer,
        cop.name(),
    ) {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

/// Flag `class X < BasePath` and autocorrect superclass to `prefer`.
pub fn check_superclass(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    base_path: &str,
    prefer: &str,
    msg: &str,
    diagnostics: &mut Vec<Diagnostic>,
    mut corrections: Option<&mut Vec<Correction>>,
) {
    if node.kind() != "class" {
        return;
    }
    let Some(superclass) = node.child_by_field_name("superclass") else {
        return;
    };
    if !matches_base(&node_text(source, superclass), base_path) {
        return;
    }
    report_superclass(
        cop,
        source,
        superclass,
        prefer,
        msg,
        diagnostics,
        &mut corrections,
    );
}
