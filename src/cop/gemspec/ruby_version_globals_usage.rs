//! Gemspec/RubyVersionGlobalsUsage — ban `RUBY_VERSION` in gemspecs.

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RubyVersionGlobalsUsage;

const NEEDLE: &str = "RUBY_VERSION";

impl Cop for RubyVersionGlobalsUsage {
    fn name(&self) -> &'static str {
        "Gemspec/RubyVersionGlobalsUsage"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*.gemspec"]
    }

    fn uses_line_phase(&self) -> bool {
        true
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        for (line_idx, line) in source.lines().enumerate() {
            let Ok(line_str) = std::str::from_utf8(line) else {
                continue;
            };
            if line_str.trim_start().starts_with('#') {
                continue;
            }
            scan_ruby_version(self, source, line_str, line_idx + 1, diagnostics);
        }
    }
}

fn scan_ruby_version(
    cop: &RubyVersionGlobalsUsage,
    source: &SourceFile,
    line_str: &str,
    line_num: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut from = 0;
    while let Some(rel) = line_str[from..].find(NEEDLE) {
        let pos = from + rel;
        from = pos + NEEDLE.len();
        if !is_bare_ident(line_str.as_bytes(), pos, NEEDLE.len()) || in_string_lit(line_str, pos) {
            continue;
        }
        diagnostics.push(cop.diagnostic(
            source,
            line_num,
            pos,
            "Do not use `RUBY_VERSION` in gemspec.".to_string(),
        ));
    }
}

fn is_bare_ident(b: &[u8], pos: usize, len: usize) -> bool {
    let left = pos == 0 || is_boundary(b[pos - 1]);
    let after = pos + len;
    let right = after >= b.len() || is_boundary(b[after]);
    left && right
}

fn is_boundary(c: u8) -> bool {
    !c.is_ascii_alphanumeric() && c != b'_'
}

/// True when `pos` is inside a quote, but not inside a `#{}` interpolation.
fn in_string_lit(line: &str, pos: usize) -> bool {
    let mut st = QuoteState::default();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < pos && i < bytes.len() {
        st.step(bytes, &mut i);
    }
    st.in_single || (st.in_double && st.interp == 0)
}

#[derive(Default)]
struct QuoteState {
    in_single: bool,
    in_double: bool,
    interp: i32,
}

impl QuoteState {
    fn step(&mut self, bytes: &[u8], i: &mut usize) {
        if self.in_double && self.interp > 0 {
            self.brace(bytes[*i]);
            *i += 1;
            return;
        }
        self.step_outer(bytes, i);
    }

    fn step_outer(&mut self, bytes: &[u8], i: &mut usize) {
        if self.toggle_quote(bytes[*i]) || self.skip_escape(bytes, i) || self.open_interp(bytes, i)
        {
            return;
        }
        *i += 1;
    }

    fn toggle_quote(&mut self, c: u8) -> bool {
        if c == b'\'' && !self.in_double {
            self.in_single ^= true;
            return false; // still advance
        }
        if c == b'"' && !self.in_single {
            self.in_double ^= true;
        }
        false
    }

    fn skip_escape(&self, bytes: &[u8], i: &mut usize) -> bool {
        if bytes[*i] == b'\\' && (self.in_single || self.in_double) {
            *i += 2;
            return true;
        }
        false
    }

    fn open_interp(&mut self, bytes: &[u8], i: &mut usize) -> bool {
        if bytes[*i] == b'#' && self.in_double && bytes.get(*i + 1) == Some(&b'{') {
            self.interp = 1;
            *i += 2;
            return true;
        }
        false
    }

    fn brace(&mut self, c: u8) {
        self.interp += i32::from(c == b'{') - i32::from(c == b'}');
    }
}
