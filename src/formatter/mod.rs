pub mod files;
pub mod github;
pub mod json;
pub mod progress;
pub mod quiet;
pub mod text;

use std::io::Write;
use std::path::PathBuf;

use crate::diagnostic::Diagnostic;

pub trait Formatter {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write);

    fn print(&self, diagnostics: &[Diagnostic], files: &[PathBuf]) {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        self.format_to(diagnostics, files, &mut lock);
    }
}

pub fn create_formatter(format: &str) -> Box<dyn Formatter> {
    match format {
        "json" => Box::new(json::JsonFormatter),
        "github" => Box::new(github::GithubFormatter),
        "quiet" => Box::new(quiet::QuietFormatter),
        "files" => Box::new(files::FilesFormatter),
        "emacs" | "simple" | "text" => Box::new(text::TextFormatter),
        _ => Box::new(progress::ProgressFormatter),
    }
}
