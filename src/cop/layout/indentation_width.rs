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
    // `when`/`in`/`else`/… alignment is CaseIndentation / ElseAlignment, not Width.
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
    matches!(
        rest,
        b"else" | b"rescue" | b"ensure"
    ) || rest.starts_with(b"when ")
        || rest.starts_with(b"when(")
        || rest.starts_with(b"in ")
        || rest.starts_with(b"in(")
        || rest.starts_with(b"elsif ")
        || rest.starts_with(b"elsif(")
        || rest.starts_with(b"else ")
        || rest.starts_with(b"rescue ")
        || rest.starts_with(b"ensure ")
}

fn ends_with_open_delim(line: &[u8]) -> bool {
    if line_has_unclosed_open(line) {
        return true;
    }
    let code = strip_line_comment(line);
    let mut i = code.len();
    while i > 0 {
        i -= 1;
        match code[i] {
            b' ' | b'\t' | b'\r' => continue,
            b'(' | b'[' | b'{' => return true,
            _ => return false,
        }
    }
    false
}

fn line_has_unclosed_open(line: &[u8]) -> bool {
    let code = strip_line_comment(line);
    let mut depth = 0i32;
    for &b in code {
        match b {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            _ => {}
        }
    }
    depth > 0
}

fn strip_line_comment(line: &[u8]) -> &[u8] {
    match crate::parse::comment_hash::first_comment_hash(line) {
        Some(i) => &line[..i],
        None => line,
    }
}

fn ends_with_multi_char_op(t: &[u8]) -> bool {
    const OPS: &[&[u8]] = &[
        b"->", b"=>", b"&&", b"||", b"==", b"!=", b">=", b"<=", b"<<", b">>",
    ];
    OPS.iter().any(|op| t.ends_with(op))
}

fn ends_with_single_char_op(t: &[u8]) -> bool {
    matches!(
        t.last(),
        Some(b',' | b'\\' | b'(' | b'[' | b'{' | b'+' | b'-' | b'*' | b'/' | b'%' | b'|' | b'^' | b'<' | b'>')
    )
}

fn ends_with_continuation(line: &[u8]) -> bool {
    let t = trim_ascii_end(strip_line_comment(line));
    !t.is_empty()
        && (ends_with_multi_char_op(t)
            || ends_with_single_char_op(t)
            || trailing_if_kw(t)
            || case_opener(t))
}

/// `x = case y` / `case y` — `when`/`end` may align under `case`, not Width steps.
fn case_opener(t: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(t) else {
        return false;
    };
    let s = s.trim_end();
    s.starts_with("case ")
        || s.starts_with("case(")
        || s.contains(" = case ")
        || s.contains("=case ")
        || s.ends_with("= case")
}

fn trim_ascii_end(code: &[u8]) -> &[u8] {
    let mut end = code.len();
    while end > 0 && matches!(code[end - 1], b' ' | b'\t' | b'\r') {
        end -= 1;
    }
    &code[..end]
}

fn trailing_if_kw(t: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(t) else {
        return false;
    };
    let s = s.trim_end();
    if s.ends_with("then") || s.ends_with("end") {
        return false;
    }
    let st = s.trim_start();
    s.contains(" if ")
        || s.contains(" unless ")
        || st.starts_with("if ")
        || st.starts_with("unless ")
        || st.starts_with("else")
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

fn aligned_continuation(
    indent: usize,
    prev: usize,
    prev_line: &[u8],
    cont_base: &mut Option<usize>,
) -> bool {
    let start = indent > prev
        && (ends_with_open_delim(prev_line) || ends_with_continuation(prev_line));
    let ongoing = cont_base.is_some_and(|b| indent >= b.saturating_sub(1));
    if start || ongoing {
        if cont_base.is_none() {
            *cont_base = Some(prev);
        }
        true
    } else {
        *cont_base = None;
        false
    }
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
            cop, source, code_map, line_no, indent, prev, width, diagnostics, corrections,
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
            if aligned_continuation(indent, prev, prev_line, &mut cont_base) {
                prev_line = line;
                continue;
            }
            check_step_from_prev(
                cop, source, code_map, i + 1, indent, prev, width, diagnostics, corrections,
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
    fn name(&self) -> &'static str { "Layout/IndentationWidth" }
    fn supports_autocorrect(&self) -> bool { true }

    fn uses_source_phase(&self) -> bool {
        true
    }

    fn check_source(
        &self, source: &SourceFile, _tree: &Tree, code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
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
