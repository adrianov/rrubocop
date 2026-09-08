//! Inspection and autocorrection logic shared by MCP tools.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::cli::AutocorrectMode;
use crate::config::{load_config, CopFilterSet, ResolvedConfig};
use crate::diagnostic::{smart_path, Diagnostic};

use super::io::{lint_mut, lint_once, read_file, target_files, write_file};
use super::offense;
use super::state::State;

pub(crate) fn inspect(
    state: &State,
    path: Option<String>,
    source: Option<String>,
) -> Result<String, String> {
    with_resolved(state, path.as_deref(), |cfg, filters| {
        inspect_resolved(state, cfg, filters, path.as_deref(), source)
    })
}

fn inspect_resolved(
    state: &State,
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    path: Option<&str>,
    source: Option<String>,
) -> Result<String, String> {
    if let Some(code) = source {
        let diags = lint_once(
            state,
            cfg,
            filters,
            Path::new(path.unwrap_or("example.rb")),
            code.as_bytes(),
        )?;
        return Ok(offense::offenses_json(&diags));
    }
    let files = target_files(filters, path)?;
    Ok(pack_offenses(&files, &lint_paths(state, cfg, filters, &files)?))
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
    with_resolved(state, path.as_deref(), |cfg, filters| {
        if let Some(code) = source {
            return correct_inline(state, cfg, filters, path.as_deref(), code, mode);
        }
        correct_files(state, cfg, filters, path.as_deref(), mode)
    })
}

fn with_resolved<T>(
    state: &State,
    path: Option<&str>,
    f: impl FnOnce(&ResolvedConfig, &CopFilterSet) -> Result<T, String>,
) -> Result<T, String> {
    if let Some(fixed) = &state.fixed {
        return f(&fixed.config, &fixed.filters);
    }
    let config = load_config(None, path.map(Path::new), None).map_err(|e| e.to_string())?;
    let filters = CopFilterSet::build(&config, &state.registry);
    f(&config, &filters)
}

fn correct_inline(
    state: &State,
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    path: Option<&str>,
    code: String,
    mode: AutocorrectMode,
) -> Result<String, String> {
    let display = path.unwrap_or("example.rb");
    let mut bytes = code.into_bytes();
    lint_mut(state, cfg, filters, Path::new(display), &mut bytes, mode)?;
    if let Some(p) = path {
        write_file(Path::new(p), &bytes)?;
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn correct_files(
    state: &State,
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    path: Option<&str>,
    mode: AutocorrectMode,
) -> Result<String, String> {
    let files = target_files(filters, path)?;
    let results: Vec<Value> = files
        .iter()
        .map(|file| correct_one(state, cfg, filters, file, mode))
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

fn correct_one(
    state: &State,
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    file: &Path,
    mode: AutocorrectMode,
) -> Result<Value, String> {
    let original = read_file(file)?;
    let mut bytes = original.clone();
    lint_mut(state, cfg, filters, file, &mut bytes, mode)?;
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
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    files: &[PathBuf],
) -> Result<Vec<(String, Vec<Diagnostic>)>, String> {
    files
        .iter()
        .map(|file| {
            let diags = lint_once(state, cfg, filters, file, &read_file(file)?)?;
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

