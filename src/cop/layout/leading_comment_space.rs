//! Layout/LeadingCommentSpace.

use tree_sitter::Tree;

use crate::cop::layout::report;
use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct LeadingCommentSpace;

fn missing_space_after_hash(text: &[u8]) -> bool {
    if text.is_empty() || text[0] != b'#' { return false; }
    if text.starts_with(b"#++") || text.starts_with(b"#--") { return false; }
    let hash_run = text.iter().take_while(|&&b| b == b'#').count();
    match text.get(hash_run) {
        None => false,
        Some(b) if b.is_ascii_whitespace() || *b == b'=' => false,
        Some(_) => true,
    }
}

fn skip_shebang(text: &[u8], line: usize, prev_line: Option<usize>, prev_shebang: bool) -> bool {
    if !text.starts_with(b"#!") { return false; }
    let cont = line > 1 && prev_line == Some(line - 1) && prev_shebang;
    line == 1 || cont
}

fn skip_config_ru(source: &SourceFile, text: &[u8], line: usize) -> bool {
    if !(text.starts_with(b"#\\") && line == 1) { return false; }
    std::path::Path::new(source.path_str()).file_name().and_then(|n| n.to_str()) == Some("config.ru")
}

fn advance(prev_line: &mut Option<usize>, prev_shebang: &mut bool, line: usize, shebang: bool) {
    *prev_line = Some(line);
    *prev_shebang = shebang;
}

fn handle_comment(
    cop: &dyn Cop, source: &SourceFile, text: &[u8], start: usize, line: usize,
    prev_line: &mut Option<usize>, prev_shebang: &mut bool,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !missing_space_after_hash(text) {
        advance(prev_line, prev_shebang, line, false);
        return;
    }
    if skip_shebang(text, line, *prev_line, *prev_shebang) {
        advance(prev_line, prev_shebang, line, true);
        return;
    }
    if skip_config_ru(source, text, line) {
        advance(prev_line, prev_shebang, line, false);
        return;
    }
    report::insert_space(
        cop, source, start, "Missing space after `#`.".into(),
        diagnostics, corrections, start + 1,
    );
    advance(prev_line, prev_shebang, line, false);
}

impl Cop for LeadingCommentSpace {
    fn name(&self) -> &'static str { "Layout/LeadingCommentSpace" }
    fn supports_autocorrect(&self) -> bool { true }

    fn check_source(
        &self, source: &SourceFile, tree: &Tree, code_map: &CodeMap,
        config: &CopConfig, diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = (code_map, config);
        let bytes = source.as_bytes();
        let mut prev_line = None::<usize>;
        let mut prev_shebang = false;
        for comment in shared::collect_comments(tree.root_node()) {
            let start = comment.start_byte();
            let text = &bytes[start..comment.end_byte()];
            let (line, _) = source.offset_to_line_col(start);
            handle_comment(
                self, source, text, start, line, &mut prev_line, &mut prev_shebang,
                diagnostics, &mut corrections,
            );
        }
    }
}
