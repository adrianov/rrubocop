//! Layout/IndentationWidth.

mod cont;

use tree_sitter::Tree;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct IndentationWidth;

fn line_indent(line: &[u8]) -> Option<usize> {
    if line.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r') {
        return None;
    }
    let indent = line.iter().take_while(|&&b| b == b' ' || b == b'\t').count();
    let rest = &line[indent..];
    if rest.starts_with(b"#")
        || rest.starts_with(b".")
        || rest.starts_with(b"&.")
        || is_branch_keyword(rest)
    {
        None
    } else {
        Some(indent)
    }
}

fn is_branch_keyword(rest: &[u8]) -> bool {
    let rest = trim_ascii_end(rest);
    matches!(rest, b"else" | b"rescue" | b"ensure")
        || rest.starts_with(b"when ")
        || rest.starts_with(b"when(")
        || rest.starts_with(b"in ")
        || rest.starts_with(b"in(")
        || rest.starts_with(b"elsif ")
        || rest.starts_with(b"elsif(")
        || rest.starts_with(b"else ")
        || rest.starts_with(b"rescue ")
        || rest.starts_with(b"ensure ")
}

fn trim_ascii_end(code: &[u8]) -> &[u8] {
    let mut end = code.len();
    while end > 0 && matches!(code[end - 1], b' ' | b'\t' | b'\r') {
        end -= 1;
    }
    &code[..end]
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
    diff != 0 && width != 0 && diff % width != 0 && indent > 0
}

fn report_width(
    cop: &dyn Cop,
    source: &SourceFile,
    code_map: &CodeMap,
    line_no: usize,
    indent: usize,
    prev: usize,
    width: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let off = source.line_start(line_no).unwrap_or(0);
    if code_map.covers(off + indent) {
        return;
    }
    let diff = indent.abs_diff(prev);
    let expected = expected_indent(indent, prev, width);
    report::report_fix(
        cop,
        source,
        off,
        format!("Use {width} (not {diff}) spaces for indentation."),
        diagnostics,
        corrections,
        off,
        off + indent,
        " ".repeat(expected),
    );
}

fn check_step_from_prev(
    cop: &dyn Cop,
    source: &SourceFile,
    code_map: &CodeMap,
    line_no: usize,
    indent: usize,
    prev: usize,
    width: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if bad_step(indent, prev, width) {
        report_width(
            cop,
            source,
            code_map,
            line_no,
            indent,
            prev,
            width,
            diagnostics,
            corrections,
        );
    }
}

fn scan_file_indents(
    cop: &dyn Cop,
    source: &SourceFile,
    code_map: &CodeMap,
    width: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let mut prev_indent: Option<usize> = None;
    let mut prev_line: &[u8] = b"";
    let mut cont_base: Option<usize> = None;
    for (i, line) in source.lines().enumerate() {
        let Some(indent) = line_indent(line) else {
            continue;
        };
        let off = source.line_start(i + 1).unwrap_or(0);
        if let Some(prev) = prev_indent {
            if cont::aligned_continuation(indent, prev, prev_line, &mut cont_base) {
                prev_line = line;
                continue;
            }
            check_step_from_prev(
                cop,
                source,
                code_map,
                i + 1,
                indent,
                prev,
                width,
                diagnostics,
                corrections,
            );
        }
        if code_map.covers(off + indent) {
            prev_line = line;
            continue;
        }
        prev_indent = Some(indent);
        prev_line = line;
    }
}

impl Cop for IndentationWidth {
    fn name(&self) -> &'static str {
        "Layout/IndentationWidth"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn uses_source_phase(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        _tree: &Tree,
        code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        scan_file_indents(
            self,
            source,
            code_map,
            config.get_usize("Width", 2),
            diagnostics,
            &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(IndentationWidth, "cops/layout/indentation_width");
}
