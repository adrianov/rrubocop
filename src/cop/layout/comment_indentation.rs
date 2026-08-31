//! Layout/CommentIndentation.

use tree_sitter::{Node, Tree};

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct CommentIndentation;

fn standalone_comment(bytes: &[u8], ls: usize, start: usize) -> bool {
    !bytes[ls..start].iter().any(|&b| b != b' ' && b != b'\t')
}

fn line_content_indent(bytes: &[u8], nls: usize) -> Option<usize> {
    let mut i = nls;
    while i < bytes.len() && bytes[i] != b'\n' {
        if bytes[i] != b' ' && bytes[i] != b'\t' && bytes[i] != b'\r' {
            if bytes[i] == b'#' {
                return None;
            }
            return Some(i - nls);
        }
        i += 1;
    }
    None
}

fn next_code_indent(source: &SourceFile, bytes: &[u8], line: usize) -> Option<usize> {
    let mut check_line = line + 1;
    while let Some(nls) = source.line_start(check_line) {
        if let Some(ind) = line_content_indent(bytes, nls) {
            return Some(ind);
        }
        check_line += 1;
        if check_line > line + 20 {
            break;
        }
    }
    None
}

fn check_comment(
    cop: &dyn Cop,
    source: &SourceFile,
    bytes: &[u8],
    comment: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let start = comment.start_byte();
    let (line, col) = source.offset_to_line_col(start);
    let ls = source.line_start(line).unwrap_or(0);
    if !standalone_comment(bytes, ls, start) {
        return;
    }
    let Some(ni) = next_code_indent(source, bytes, line) else {
        return;
    };
    if col == ni {
        return;
    }
    report::report_fix(
        cop,
        source,
        start,
        format!("Incorrect indentation detected (column {col} instead of {ni})."),
        diagnostics,
        corrections,
        ls,
        start,
        " ".repeat(ni),
    );
}

impl Cop for CommentIndentation {
    fn name(&self) -> &'static str {
        "Layout/CommentIndentation"
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
        let _ = (code_map, config);
        let bytes = source.as_bytes();
        for comment in shared::collect_comments(tree.root_node()) {
            check_comment(self, source, bytes, comment, diagnostics, &mut corrections);
        }
    }
}
