//! Bundler/GemFilename — Gemfile vs gems.rb naming.

use std::path::Path;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct GemFilename;

impl Cop for GemFilename {
    fn name(&self) -> &'static str {
        "Bundler/GemFilename"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/Gemfile", "**/gems.rb"]
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "Gemfile");
        let file_name = Path::new(source.path_str())
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if let Some(msg) = mismatch_msg(style, file_name, source.path_str()) {
            diagnostics.push(self.diagnostic(source, 1, 0, msg));
        }
    }
}

fn mismatch_msg(style: &str, file_name: &str, path: &str) -> Option<String> {
    match (style, file_name) {
        ("Gemfile", "gems.rb") => Some(format!(
            "`gems.rb` file was found but `Gemfile` is required (file path: {path})."
        )),
        ("Gemfile", "gems.locked") => Some(format!(
            "Expected a `Gemfile.lock` with `Gemfile` but found `gems.locked` file (file path: {path})."
        )),
        ("gems.rb", "Gemfile") => Some(format!(
            "`Gemfile` was found but `gems.rb` is required (file path: {path})."
        )),
        ("gems.rb", "Gemfile.lock") => Some(format!(
            "Expected a `gems.locked` with `gems.rb` but found `Gemfile.lock` file (file path: {path})."
        )),
        _ => None,
    }
}
