//! Local gem root via `bundle info --path` for private / non-vendored gems.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::SystemTime;

use anyhow::{Context, Result};

use super::gem_path;

struct PathCache {
    entries: HashMap<(PathBuf, String), PathBuf>,
    lockfile_mtime: Option<SystemTime>,
    working_dir: PathBuf,
}

static PATH_CACHE: Mutex<Option<PathCache>> = Mutex::new(None);

fn lockfile_mtime(working_dir: &Path) -> Option<SystemTime> {
    for name in &["Gemfile.lock", "gems.locked"] {
        if let Ok(m) = working_dir.join(name).metadata().and_then(|m| m.modified()) {
            return Some(m);
        }
    }
    None
}

fn has_lockfile(working_dir: &Path) -> bool {
    working_dir.join("Gemfile.lock").exists() || working_dir.join("gems.locked").exists()
}

fn cached_path(working_dir: &Path, gem_name: &str, mtime: Option<SystemTime>) -> Option<PathBuf> {
    let cache = PATH_CACHE.lock().unwrap();
    let c = cache.as_ref()?;
    if c.working_dir != working_dir || c.lockfile_mtime != mtime {
        return None;
    }
    c.entries
        .get(&(working_dir.to_path_buf(), gem_name.to_string()))
        .cloned()
}

fn store_path(working_dir: &Path, gem_name: &str, mtime: Option<SystemTime>, path: PathBuf) {
    let mut cache = PATH_CACHE.lock().unwrap();
    let c = cache.get_or_insert_with(|| PathCache {
        entries: HashMap::new(),
        lockfile_mtime: mtime,
        working_dir: working_dir.to_path_buf(),
    });
    if c.lockfile_mtime != mtime || c.working_dir != working_dir {
        c.entries.clear();
        c.lockfile_mtime = mtime;
        c.working_dir = working_dir.to_path_buf();
    }
    c.entries
        .insert((working_dir.to_path_buf(), gem_name.to_string()), path);
}

fn run_bundle_info(gem_name: &str, working_dir: &Path) -> Result<std::process::Output> {
    if gem_path::needs_mise_exec(working_dir) {
        Command::new("mise")
            .args(["exec", "--", "bundle", "info", "--path", gem_name])
            .current_dir(working_dir)
            .output()
            .context("mise exec -- bundle info failed")
    } else {
        Command::new("bundle")
            .args(["info", "--path", gem_name])
            .current_dir(working_dir)
            .output()
            .context("bundle not found on PATH")
    }
}

fn root_from_bundle_output(
    gem_name: &str,
    working_dir: &Path,
    output: std::process::Output,
) -> Result<PathBuf> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "gem '{gem_name}' not in bundle at {}: {}",
            working_dir.display(),
            stderr.trim()
        );
    }
    let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    if path.exists() {
        return Ok(path);
    }
    anyhow::bail!(
        "gem '{gem_name}' resolved to {} but path missing",
        path.display()
    )
}

/// Resolve a gem install root with `bundle info --path` (cached per lockfile).
///
/// Used only when the gem is not in the vendored embed set (e.g. private
/// company style gems). Public RuboCop plugins should stay vendored.
pub fn resolve_local_root(gem_name: &str, working_dir: &Path) -> Result<PathBuf> {
    if !has_lockfile(working_dir) {
        anyhow::bail!("no Gemfile.lock in {}", working_dir.display());
    }
    let mtime = lockfile_mtime(working_dir);
    if let Some(path) = cached_path(working_dir, gem_name, mtime) {
        return Ok(path);
    }
    let path = root_from_bundle_output(gem_name, working_dir, run_bundle_info(gem_name, working_dir)?)?;
    store_path(working_dir, gem_name, mtime, path.clone());
    Ok(path)
}
