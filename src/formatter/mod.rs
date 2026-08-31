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
///
/// Marks that arrive before [`Self::started`] (discovery still walking) are
/// buffered and flushed with the "Inspecting N files" header.
pub struct ProgressSink<'a> {
    fmt: &'a dyn Formatter,
    out: Mutex<Stdout>,
    state: Mutex<SinkState>,
}

struct SinkState {
    started: bool,
    pending: Vec<Vec<Diagnostic>>,
}

impl<'a> ProgressSink<'a> {
    pub fn new(fmt: &'a dyn Formatter) -> Self {
        Self {
            fmt,
            out: Mutex::new(std::io::stdout()),
            state: Mutex::new(SinkState {
                started: false,
                pending: Vec::new(),
            }),
        }
    }

    pub fn started(&self, file_count: usize) {
        let mut state = self.state.lock().unwrap();
        let mut out = self.out.lock().unwrap();
        self.fmt.started(file_count, &mut *out);
        for diags in state.pending.drain(..) {
            self.fmt.file_finished(&diags, &mut *out);
        }
        state.started = true;
    }

    pub fn file_finished(&self, diagnostics: &[Diagnostic]) {
        let mut state = self.state.lock().unwrap();
        if !state.started {
            state.pending.push(diagnostics.to_vec());
            return;
        }
        drop(state);
        self.fmt
            .file_finished(diagnostics, &mut *self.out.lock().unwrap());
    }

    pub fn finished(&self, diagnostics: &[Diagnostic], files: &[PathBuf]) {
        // Discovery should have called started; flush any stragglers just in case.
        {
            let mut state = self.state.lock().unwrap();
            if !state.started {
                let n = state.pending.len();
                let mut out = self.out.lock().unwrap();
                self.fmt.started(n, &mut *out);
                for diags in state.pending.drain(..) {
                    self.fmt.file_finished(&diags, &mut *out);
                }
                state.started = true;
            }
        }
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
