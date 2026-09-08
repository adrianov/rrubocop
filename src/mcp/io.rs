//! File discovery and lint application used by MCP inspect/autocorrect.

use std::path::{Path, PathBuf};

use crate::cli::AutocorrectMode;
use crate::config::{CopFilterSet, ResolvedConfig};
use crate::diagnostic::Diagnostic;
use crate::fs;
use crate::linter::lint_bytes_autocorrect;

use super::state::State;
use super::targets;

pub(super) fn lint_once(
    state: &State,
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    path: &Path,
    bytes: &[u8],
) -> Result<Vec<Diagnostic>, String> {
    lint_mut(state, cfg, filters, path, &mut bytes.to_vec(), AutocorrectMode::Off)
}

pub(super) fn lint_mut(
    state: &State,
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    path: &Path,
    bytes: &mut Vec<u8>,
    mode: AutocorrectMode,
) -> Result<Vec<Diagnostic>, String> {
    lint_bytes_autocorrect(
        path,
        bytes,
        cfg,
        &state.registry,
        filters,
        None,
        &[],
        mode,
        false,
    )
    .map_err(|e| e.to_string())
}

pub(super) fn target_files(filters: &CopFilterSet, targets: &[String]) -> Result<Vec<PathBuf>, String> {
    targets::validate_roots(targets)?;
    fs::discover_files_filtered(
        &targets.iter().map(PathBuf::from).collect::<Vec<_>>(),
        filters,
        false,
    )
    .map(|d| d.files)
    .map_err(|e| e.to_string())
}

pub(super) fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|_| format!("No such file or directory: {}", path.display()))
}

pub(super) fn write_file(path: &Path, content: &[u8]) -> Result<(), String> {
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
