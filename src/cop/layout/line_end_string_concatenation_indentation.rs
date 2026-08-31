//! Layout/LineEndStringConcatenationIndentation.

use tree_sitter::Tree;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct LineEndStringConcatenationIndentation;

fn is_cont(bytes: &[u8], i: usize) -> bool {
    bytes[i] == b'\\' && matches!(bytes.get(i + 1), Some(b'\n') | Some(b'\r'))
}

fn concat_msg(style: &str) -> String {
    if style == "aligned" {
        "Align parts of a string concatenated with backslash.".into()
    } else {
        "Indent the first part of a string concatenated with backslash.".into()
    }
}

fn expected_indent(base: usize, width: usize, style: &str) -> usize {
    if style == "indented" {
        base + width
    } else {
        base
    }
}

fn next_indent(source: &SourceFile, i: usize) -> Option<(usize, usize)> {
    let (line, _) = source.offset_to_line_col(i);
    let ls = source.line_start(line + 1)?;
    Some((ls, shared::line_indent(source, ls)))
}

fn check_cont(
    cop: &dyn Cop,
    source: &SourceFile,
    code_map: &CodeMap,
    i: usize,
    width: usize,
    style: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !code_map.covers(i.saturating_sub(1)) {
        return;
    }
    let Some((ls, actual)) = next_indent(source, i) else {
        return;
    };
    let expected = expected_indent(shared::line_indent(source, i), width, style);
    if actual == expected || !code_map.covers(ls + actual.min(1)) {
        return;
    }
    report::report_fix(
        cop,
        source,
        ls,
        concat_msg(style),
        diagnostics,
        corrections,
        ls,
        ls + actual,
        " ".repeat(expected),
    );
}

impl Cop for LineEndStringConcatenationIndentation {
    fn name(&self) -> &'static str {
        "Layout/LineEndStringConcatenationIndentation"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = tree;
        let width = config.get_usize("IndentationWidth", 2);
        let style = config.get_str("EnforcedStyle", "aligned");
        let bytes = source.as_bytes();
        let mut i = 0;
        while i + 1 < bytes.len() {
            if is_cont(bytes, i) {
                check_cont(
                    self,
                    source,
                    code_map,
                    i,
                    width,
                    style,
                    diagnostics,
                    &mut corrections,
                );
            }
            i += 1;
        }
    }
}
