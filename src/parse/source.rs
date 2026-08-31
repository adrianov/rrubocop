use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::diagnostic::Location;

#[derive(Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: Vec<u8>,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn from_path(path: &Path) -> Result<Self> {
        let content =
            std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        Ok(Self::from_bytes(path, content))
    }

    pub fn from_bytes(path: impl Into<PathBuf>, content: Vec<u8>) -> Self {
        let line_starts = compute_line_starts(&content);
        Self {
            path: path.into(),
            content,
            line_starts,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.content
    }

    pub fn path_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    pub fn lines(&self) -> impl Iterator<Item = &[u8]> {
        self.content.split(|&b| b == b'\n')
    }

    /// Number of lines (including a trailing empty line after a final `\n`).
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn offset_to_line_col(&self, byte_offset: usize) -> (usize, usize) {
        let line_idx = match self.line_starts.binary_search(&byte_offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_bytes = &self.content[self.line_starts[line_idx]..byte_offset];
        let col = line_bytes.iter().filter(|&&b| (b & 0xC0) != 0x80).count();
        (line_idx + 1, col)
    }

    pub fn line_start(&self, line: usize) -> Option<usize> {
        if line == 0 || line > self.line_starts.len() {
            None
        } else {
            Some(self.line_starts[line - 1])
        }
    }

    /// Byte offset of (1-based line, 0-based display column). Best-effort for ASCII.
    pub fn line_col_to_offset(&self, line: usize, column: usize) -> Option<usize> {
        let start = self.line_start(line)?;
        Some(start + column)
    }

    pub fn location_at(&self, byte_offset: usize) -> Location {
        let (line, column) = self.offset_to_line_col(byte_offset);
        Location { line, column }
    }
}

fn compute_line_starts(content: &[u8]) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, &b) in content.iter().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}
