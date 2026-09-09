//! Inspection and autocorrection logic shared by MCP tools.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::cli::AutocorrectMode;
use crate::config::{load_config, load_default_config, CopFilterSet, ResolvedConfig};
use crate::diagnostic::{smart_path, Diagnostic};

use super::io::{lint_mut, lint_once, read_file, target_files, write_file};
use super::offense;
use super::state::State;
use super::targets;

pub(crate) fn inspect(
    state: &State,
    targets: Vec<String>,
    source: Option<String>,
) -> Result<String, String> {
    if source.is_none() {
        targets::validate_roots(&targets)?;
    }
    with_resolved(state, targets.first().map(String::as_str), |cfg, filters| {
        inspect_resolved(state, cfg, filters, &targets, source)
    })
}

fn inspect_resolved(
    state: &State,
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    targets: &[String],
    source: Option<String>,
) -> Result<String, String> {
    if let Some(code) = source {
        let diags = lint_once(
            state,
            cfg,
            filters,
            Path::new(targets.first().map(String::as_str).unwrap_or("example.rb")),
            code.as_bytes(),
        )?;
        return Ok(offense::offenses_json(&diags));
    }
    let files = target_files(filters, targets)?;
    Ok(pack_offenses(&files, &lint_paths(state, cfg, filters, &files)?))
}

pub(crate) fn autocorrect(
    state: &State,
    targets: Vec<String>,
    source: Option<String>,
    safety: bool,
) -> Result<String, String> {
    let mode = if safety {
        AutocorrectMode::Safe
    } else {
        AutocorrectMode::All
    };
    if source.is_none() {
        targets::validate_roots(&targets)?;
    }
    with_resolved(state, targets.first().map(String::as_str), |cfg, filters| {
        if let Some(code) = source {
            return correct_inline(state, cfg, filters, &targets, code, mode);
        }
        correct_files(state, cfg, filters, &targets, mode)
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
    // No target path: built-in defaults only. Never walk MCP cwd (often $HOME).
    let config = match path {
        None => load_default_config(None, None),
        Some(p) => {
            targets::refuse_home_walk(Path::new(p))?;
            load_config(None, Some(Path::new(p)), None).map_err(|e| format!("{e:#}"))?
        }
    };
    f(&config, &CopFilterSet::build(&config, &state.registry))
}

fn correct_inline(
    state: &State,
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    targets: &[String],
    code: String,
    mode: AutocorrectMode,
) -> Result<String, String> {
    let mut bytes = code.into_bytes();
    lint_mut(
        state,
        cfg,
        filters,
        Path::new(targets.first().map(String::as_str).unwrap_or("example.rb")),
        &mut bytes,
        mode,
    )?;
    if let Some(p) = targets.first() {
        write_file(Path::new(p), &bytes)?;
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn correct_files(
    state: &State,
    cfg: &ResolvedConfig,
    filters: &CopFilterSet,
    targets: &[String],
    mode: AutocorrectMode,
) -> Result<String, String> {
    let files = target_files(filters, targets)?;
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
