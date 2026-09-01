//! Style/CommentAnnotation — breadth-first tree-sitter port.

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CommentAnnotation;

impl Cop for CommentAnnotation {
    fn name(&self) -> &'static str {
        "Style/CommentAnnotation"
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let require_colon = config.get_bool("RequireColon", true);
        let lines: Vec<String> = source
            .lines()
            .map(|l| String::from_utf8_lossy(l).into_owned())
            .collect();
        for (idx, line) in lines.iter().enumerate() {
            if !first_comment_or_inline(&lines, idx, line) {
                continue;
            }
            check_line(self, source, idx, line, require_colon, diagnostics);
        }
    }
}

/// RuboCop only registers on the first line of a contiguous comment block
/// (or an inline trailing comment), so annotations mid-paragraph are ignored.
fn first_comment_or_inline(lines: &[String], idx: usize, line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return false;
    }
    // Inline: code before `#` on the same line.
    let before = &line[..line.len() - trimmed.len()];
    if before.chars().any(|c| !c.is_whitespace()) {
        return true;
    }
    if idx == 0 {
        return true;
    }
    let prev = lines[idx - 1].trim_start();
    !prev.starts_with('#')
}

fn check_line(
    cop: &CommentAnnotation,
    source: &SourceFile,
    idx: usize,
    line: &str,
    require_colon: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let trimmed = line.trim_start();
    // RuboCop AnnotationComment: `/^(# ?)(KEYWORD)/` — at most one space after `#`.
    let body = if let Some(rest) = trimmed.strip_prefix("# ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix('#') {
        if rest.starts_with(char::is_whitespace) {
            return;
        }
        rest
    } else {
        return;
    };
    for kw in ["TODO", "FIXME", "OPTIMIZE", "HACK", "REVIEW", "NOTE"] {
        if let Some(msg) = annotation_msg(body, kw, require_colon) {
            let (line_n, col) =
                source.offset_to_line_col(source.line_start(idx + 1).unwrap_or(0));
            diagnostics.push(cop.diagnostic(source, line_n, col, msg));
            return;
        }
    }
}

fn annotation_msg(body: &str, kw: &str, require_colon: bool) -> Option<String> {
    if !body.starts_with(kw) {
        return None;
    }
    let rest = &body[kw.len()..];
    // RuboCop AnnotationComment: keyword + (colon || space); note required for style msgs.
    let (has_colon, after) = if let Some(r) = rest.strip_prefix(':') {
        (true, r)
    } else if rest.starts_with(char::is_whitespace) {
        (false, rest.trim_start())
    } else {
        return None;
    };
    if after.is_empty() {
        return None;
    }
    // Lowercase keyword sentence ("Todo foo") is not an annotation offense path here;
    // we only match uppercase keywords from the list.
    if require_colon == has_colon {
        // Also need space after colon when require_colon.
        if has_colon && !rest.starts_with(": ") && rest.starts_with(':') {
            return Some(format!(
                "Annotation keywords like `{kw}` should be followed by a colon."
            ));
        }
        return None;
    }
    Some(format!(
        "Annotation keywords like `{kw}` should be followed by a colon."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(CommentAnnotation, "cops/style/comment_annotation");
}
