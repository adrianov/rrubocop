//! Helpers for Layout/SpaceAroundKeyword.

use tree_sitter::Node;

use crate::cop::Cop;
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub const KEYWORDS: &[&[u8]] = &[
    b"and", b"or", b"not", b"if", b"unless", b"while", b"until", b"for", b"in", b"do", b"then",
    b"when", b"else", b"elsif", b"begin", b"rescue", b"ensure", b"end", b"case", b"next", b"break",
    b"return", b"super", b"yield", b"defined?",
];

const NAMED_KW: &[&str] = &[
    "if", "unless", "while", "until", "for", "case", "when", "else", "elsif", "begin", "rescue",
    "ensure", "do_block",
];

fn is_kw_kind(k: &str) -> bool {
    KEYWORDS.iter().any(|&kw| kw == k.as_bytes())
        || matches!(k, "return" | "break" | "next" | "yield" | "super")
}

fn is_anon_kw(bytes: &[u8], n: Node<'_>) -> bool {
    let t = &bytes[n.start_byte()..n.end_byte()];
    !n.is_named() && KEYWORDS.iter().any(|&kw| t == kw)
}

pub fn should_check(bytes: &[u8], n: Node<'_>) -> bool {
    let k = n.kind();
    is_kw_kind(k) || is_anon_kw(bytes, n) || NAMED_KW.contains(&k)
}

fn do_end(n: Node<'_>) -> Option<Node<'_>> {
    let mut cur = n.walk();
    n.children(&mut cur).find(|c| c.kind() == "do")
}

fn named_kw_end(bytes: &[u8], n: Node<'_>, start: usize, k: &str) -> Option<usize> {
    let mut cur = n.walk();
    let c = n.children(&mut cur).next()?;
    if KEYWORDS
        .iter()
        .any(|&kw| &bytes[c.start_byte()..c.end_byte()] == kw)
    {
        Some(c.end_byte())
    } else if KEYWORDS.iter().any(|&kw| k.as_bytes() == kw) {
        Some(start + k.len())
    } else {
        None
    }
}

pub fn kw_span(bytes: &[u8], n: Node<'_>) -> Option<(usize, usize)> {
    let k = n.kind();
    let start = n.start_byte();
    if k == "do_block" {
        let d = do_end(n)?;
        return Some((d.start_byte(), d.end_byte()));
    }
    if !n.is_named() {
        return Some((start, n.end_byte()));
    }
    let end = named_kw_end(bytes, n, start, k)?;
    Some((start, end))
}

fn need_space_after(next: u8) -> bool {
    next.is_ascii_alphanumeric()
        || matches!(next, b'_' | b'(' | b':' | b'"' | b'\'' | b'@' | b'$' | b'[')
}

fn skip_after_punct(next: u8) -> bool {
    next.is_ascii_whitespace() || matches!(next, b';' | b',' | b')' | b']' | b'}')
}

fn missing_after(bytes: &[u8], k: &str, end: usize) -> bool {
    let Some(&next) = bytes.get(end) else {
        return false;
    };
    if skip_after_punct(next) {
        return false;
    }
    if k == "defined?" && next == b'(' {
        return false;
    }
    need_space_after(next) || next.is_ascii_graphic()
}

fn missing_before(bytes: &[u8], kw_start: usize) -> bool {
    if kw_start == 0 {
        return false;
    }
    let prev = bytes[kw_start - 1];
    if prev.is_ascii_whitespace() || matches!(prev, b'(' | b'{' | b'[' | b';' | b'!') {
        return false;
    }
    prev.is_ascii_alphanumeric()
        || matches!(prev, b'_' | b')' | b']' | b'}' | b'?' | b'!')
}

fn report_space(
    cop: &dyn Cop,
    source: &SourceFile,
    at: usize,
    insert_at: usize,
    msg: String,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (l, c) = source.offset_to_line_col(at);
    let mut diag = cop.diagnostic(source, l, c, msg);
    if let Some(corr) = corrections {
        corr.push(Correction {
            start: insert_at,
            end: insert_at,
            replacement: " ".into(),
            cop_name: cop.name(),
            cop_index: 0,
        });
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

pub fn check_after(
    cop: &dyn Cop,
    source: &SourceFile,
    bytes: &[u8],
    k: &str,
    kw_start: usize,
    end: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !missing_after(bytes, k, end) {
        return;
    }
    let kw = String::from_utf8_lossy(&bytes[kw_start..end]);
    report_space(
        cop,
        source,
        kw_start,
        end,
        format!("Space after keyword `{kw}` is missing."),
        diagnostics,
        corrections,
    );
}

pub fn check_before(
    cop: &dyn Cop,
    source: &SourceFile,
    bytes: &[u8],
    kw_start: usize,
    end: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !missing_before(bytes, kw_start) {
        return;
    }
    let kw = String::from_utf8_lossy(&bytes[kw_start..end]);
    report_space(
        cop,
        source,
        kw_start,
        kw_start,
        format!("Space before keyword `{kw}` is missing."),
        diagnostics,
        corrections,
    );
}
