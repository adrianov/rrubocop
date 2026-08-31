//! Layout/IndentationWidth.

use tree_sitter::Tree;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct IndentationWidth;

fn line_indent(line: &[u8]) -> Option<usize> {
    if line.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r') { return None; }
    let indent = line.iter().take_while(|&&b| b == b' ' || b == b'\t').count();
    let rest = &line[indent..];
    // Leading-dot continuations are Layout/MultilineMethodCallIndentation's job.
    if rest.starts_with(b"#") || rest.starts_with(b".") || rest.starts_with(b"&.") {
        None
    } else {
        Some(indent)
    }
}

fn ends_with_open_delim(line: &[u8]) -> bool {
    let mut i = line.len();
    while i > 0 {
        i -= 1;
        match line[i] {
            b' ' | b'\t' | b'\r' => continue,
            b'(' | b'[' | b'{' => return true,
            _ => return false,
        }
    }
    false
}

fn expected_indent(indent: usize, prev: usize, width: usize) -> usize {
    if indent > prev {
        prev + width
    } else if prev >= width {
        prev - width
    } else {
        0
    }
}

fn bad_step(indent: usize, prev: usize, width: usize) -> bool {
    let diff = indent.abs_diff(prev);
    // Allow any multiple of Width (block outdent/indent may span levels).
    diff != 0 && width != 0 && diff % width != 0 && indent > 0
}

fn report_width(
    cop: &dyn Cop, source: &SourceFile, code_map: &CodeMap, line_no: usize,
    indent: usize, prev: usize, width: usize,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let off = source.line_start(line_no).unwrap_or(0);
    if code_map.covers(off + indent) { return; }
    let diff = indent.abs_diff(prev);
    let expected = expected_indent(indent, prev, width);
    report::report_fix(
        cop, source, off,
        format!("Use {width} (not {diff}) spaces for indentation."),
        diagnostics, corrections, off, off + indent, " ".repeat(expected),
    );
}

impl Cop for IndentationWidth {
    fn name(&self) -> &'static str { "Layout/IndentationWidth" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, _tree: &Tree, code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let width = config.get_usize("Width", 2);
        let mut prev_indent: Option<usize> = None;
        let mut prev_line: &[u8] = b"";
        for (i, line) in source.lines().enumerate() {
            let Some(indent) = line_indent(line) else { continue; };
            if let Some(prev) = prev_indent {
                // RuboCop IndentationWidth is AST-based; aligned content after `(`/`[`/`{`
                // is not a Width step (e.g. `params: (\n               …`).
                let aligned_after_open = indent > prev && ends_with_open_delim(prev_line);
                if !aligned_after_open && bad_step(indent, prev, width) {
                    report_width(
                        self, source, code_map, i + 1, indent, prev, width,
                        diagnostics, &mut corrections,
                    );
                }
            }
            prev_indent = Some(indent);
            prev_line = line;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(IndentationWidth, "cops/layout/indentation_width");
}
