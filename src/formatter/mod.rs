pub mod color;
pub mod files;
pub mod github;
pub mod json;
pub mod progress;
pub mod quiet;
pub mod text;

use std::io::{Stdout, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::diagnostic::Diagnostic;

use self::color::Color;

pub trait Formatter: Sync {
    fn format_to(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write);

    /// Progress-style formatters emit marks as files finish (order free).
    fn streams_marks(&self) -> bool {
        false
    }

    fn started(&self, _file_count: usize, _out: &mut dyn Write) {}

    fn file_finished(&self, _diagnostics: &[Diagnostic], _out: &mut dyn Write) {}

    /// Trailing report after all files (marks already streamed when `streams_marks`).
    fn finished(&self, diagnostics: &[Diagnostic], files: &[PathBuf], out: &mut dyn Write) {
        self.format_to(diagnostics, files, out);
    }

    fn print(&self, diagnostics: &[Diagnostic], files: &[PathBuf]) {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        self.format_to(diagnostics, files, &mut lock);
    }
}

/// Locked stdout sink for streaming progress marks from parallel workers.
pub struct ProgressSink<'a> {
    fmt: &'a dyn Formatter,
    out: Mutex<Stdout>,
}

impl<'a> ProgressSink<'a> {
    pub fn new(fmt: &'a dyn Formatter) -> Self {
        Self {
            fmt,
            out: Mutex::new(std::io::stdout()),
        }
    }

    pub fn started(&self, file_count: usize) {
        self.fmt.started(file_count, &mut *self.out.lock().unwrap());
    }

    pub fn file_finished(&self, diagnostics: &[Diagnostic]) {
        self.fmt
            .file_finished(diagnostics, &mut *self.out.lock().unwrap());
    }

    pub fn finished(&self, diagnostics: &[Diagnostic], files: &[PathBuf]) {
        self.fmt
            .finished(diagnostics, files, &mut *self.out.lock().unwrap());
    }
}

pub fn create_formatter(format: &str, color: Color) -> Box<dyn Formatter> {
    match format {
        "json" => Box::new(json::JsonFormatter),
        "github" => Box::new(github::GithubFormatter),
        "quiet" => Box::new(quiet::QuietFormatter { color }),
        "files" => Box::new(files::FilesFormatter),
        "emacs" | "simple" | "text" => Box::new(text::TextFormatter { color }),
        _ => Box::new(progress::ProgressFormatter { color }),
    }
}
