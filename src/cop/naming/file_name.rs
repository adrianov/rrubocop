//! Naming/FileName — path basenames should be snake_case.

use std::path::Path;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FileName;

const ALLOWED: &[&str] = &[
    "Gemfile",
    "Rakefile",
    "Capfile",
    "Vagrantfile",
    "Guardfile",
    "Procfile",
];

impl Cop for FileName {
    fn name(&self) -> &'static str {
        "Naming/FileName"
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(file_name) = bad_filename(source.path_str()) else {
            return;
        };
        diagnostics.push(self.diagnostic(
            source,
            1,
            0,
            format!("The file name `{file_name}` should use snake_case."),
        ));
    }
}

fn bad_filename(path_str: &str) -> Option<&str> {
    let path = Path::new(path_str);
    let file_name = path.file_name()?.to_str()?;
    if ALLOWED.contains(&file_name) {
        return None;
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);
    stem.split('.').any(|seg| !is_filename_snake_case(seg)).then_some(file_name)
}

fn is_filename_snake_case(segment: &str) -> bool {
    segment.chars().all(|ch| {
        if ch.is_ascii() {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '?' | '!')
        } else {
            ch.is_lowercase()
        }
    })
}
