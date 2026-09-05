//! Inspection and autocorrection logic shared by MCP tools.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::cli::AutocorrectMode;
use crate::config::{CopFilterSet, ResolvedConfig};
use crate::cop::registry::CopRegistry;
use crate::diagnostic::{smart_path, Diagnostic};
use crate::fs;
use crate::linter::lint_bytes_autocorrect;

use super::offense;

pub(crate) struct State {
    pub(crate) config: ResolvedConfig,
    pub(crate) registry: CopRegistry,
    pub(crate) filters: CopFilterSet,
}

pub(crate) fn inspect(
    state: &State,
    path: Option<String>,
    source: Option<String>,
) -> Result<String, String> {
    if let Some(code) = source {
        let diags = lint_once(
            state,
            Path::new(path.as_deref().unwrap_or("example.rb")),
            code.as_bytes(),
        )?;
        return Ok(offense::offenses_json(&diags));
    }
    let files = target_files(state, path)?;
    Ok(pack_offenses(&files, &lint_paths(state, &files)?))
}

pub(crate) fn autocorrect(
    state: &State,
    path: Option<String>,
    source: Option<String>,
    safety: bool,
) -> Result<String, String> {
    let mode = if safety {
        AutocorrectMode::Safe
    } else {
        AutocorrectMode::All
    };
    if let Some(code) = source {
        return correct_inline(state, path, code, mode);
    }
    correct_files(state, path, mode)
}

fn correct_inline(
    state: &State,
    path: Option<String>,
    code: String,
    mode: AutocorrectMode,
) -> Result<String, String> {
    let display = path.clone().unwrap_or_else(|| "example.rb".into());
    let mut bytes = code.into_bytes();
    lint_mut(state, Path::new(&display), &mut bytes, mode)?;
    if let Some(p) = path.as_ref() {
        write_file(Path::new(p), &bytes)?;
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn correct_files(
    state: &State,
    path: Option<String>,
    mode: AutocorrectMode,
) -> Result<String, String> {
    let files = target_files(state, path)?;
    let results: Vec<Value> = files
        .iter()
        .map(|file| correct_one(state, file, mode))
        .collect::<Result<_, _>>()?;
    let corrected_n = results.iter().filter(|r| r["corrected"] == true).count();
    Ok(json!({
        "files": results,
        "summary": {
            "target_file_count": files.len(),
            "corrected_file_count": corrected_n
        }
    })
    .to_string())
}

fn correct_one(state: &State, file: &Path, mode: AutocorrectMode) -> Result<Value, String> {
    let original = read_file(file)?;
    let mut bytes = original.clone();
    lint_mut(state, file, &mut bytes, mode)?;
    let changed = bytes != original;
    if changed {
        write_file(file, &bytes)?;
    }
    Ok(json!({
        "path": smart_path(&file.to_string_lossy()),
        "corrected": changed
    }))
}

fn lint_paths(
    state: &State,
    files: &[PathBuf],
) -> Result<Vec<(String, Vec<Diagnostic>)>, String> {
    files
        .iter()
        .map(|file| {
            let diags = lint_once(state, file, &read_file(file)?)?;
            Ok((smart_path(&file.to_string_lossy()), diags))
        })
        .collect()
}

fn pack_offenses(targets: &[PathBuf], all: &[(String, Vec<Diagnostic>)]) -> String {
    let offense_count: usize = all.iter().map(|(_, d)| d.len()).sum();
    let files: Vec<Value> = all
        .iter()
        .filter(|(_, d)| !d.is_empty())
        .map(|(path, diags)| {
            json!({
                "path": path,
                "offenses": diags.iter().map(offense::to_lsp_offense).collect::<Vec<_>>()
            })
        })
        .collect();
    json!({
        "files": files,
        "summary": {
            "target_file_count": targets.len(),
            "offense_count": offense_count
        }
    })
    .to_string()
}

fn lint_once(state: &State, path: &Path, bytes: &[u8]) -> Result<Vec<Diagnostic>, String> {
    lint_mut(state, path, &mut bytes.to_vec(), AutocorrectMode::Off)
}

fn lint_mut(
    state: &State,
    path: &Path,
    bytes: &mut Vec<u8>,
    mode: AutocorrectMode,
) -> Result<Vec<Diagnostic>, String> {
    lint_bytes_autocorrect(
        path,
        bytes,
        &state.config,
        &state.registry,
        &state.filters,
        None,
        &[],
        mode,
        false,
    )
    .map_err(|e| e.to_string())
}

fn target_files(state: &State, path: Option<String>) -> Result<Vec<PathBuf>, String> {
    let root = path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    if !root.exists() {
        return Err(format!("No such file or directory: {}", root.display()));
    }
    fs::discover_files_filtered(&[root], &state.filters, false)
        .map(|d| d.files)
        .map_err(|e| e.to_string())
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|_| format!("No such file or directory: {}", path.display()))
}

fn write_file(path: &Path, content: &[u8]) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| write_err(path, &e))
}

fn write_err(path: &Path, e: &std::io::Error) -> String {
    use std::io::ErrorKind;
    match e.kind() {
        ErrorKind::PermissionDenied => format!("Permission denied: {}", path.display()),
        ErrorKind::StorageFull => format!("No space left on device: {}", path.display()),
        _ if e.raw_os_error() == Some(libc::EROFS) => {
            format!("Read-only file system: {}", path.display())
        }
        _ => format!("{}: {}", e, path.display()),
    }
}
