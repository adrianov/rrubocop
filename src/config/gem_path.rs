use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Mutex;
use std::time::SystemTime;

use anyhow::{Context, Result};

/// Cache for gem path resolution. Keyed on (working_dir, gem_name), stores
/// the resolved path. Invalidated when Gemfile.lock mtime changes.
struct GemPathCache {
    entries: HashMap<(PathBuf, String), PathBuf>,
    lockfile_mtime: Option<SystemTime>,
    working_dir: PathBuf,
}

static GEM_PATH_CACHE: Mutex<Option<GemPathCache>> = Mutex::new(None);

/// Resolve a gem's install path via `bundle info --path <gem_name>`.
///
/// `working_dir` is the directory where `bundle` should run (typically the
/// project root where `Gemfile.lock` lives). Results are cached per
/// (working_dir, gem_name) and invalidated when Gemfile.lock mtime changes.
pub fn resolve_gem_path(gem_name: &str, working_dir: &Path) -> Result<PathBuf> {
    let lockfile_mtime = lockfile_mtime(working_dir);
    if let Some(path) = cache_lookup(gem_name, working_dir, lockfile_mtime) {
        return Ok(path);
    }

    let output = timed_bundle_info(gem_name, working_dir)?;
    let path = parse_bundle_path(gem_name, working_dir, &output)?;
    update_cache(gem_name, working_dir, lockfile_mtime, path.clone());
    Ok(path)
}

fn lockfile_mtime(working_dir: &Path) -> Option<SystemTime> {
    working_dir
        .join("Gemfile.lock")
        .metadata()
        .and_then(|m| m.modified())
        .ok()
}

fn timed_bundle_info(gem_name: &str, working_dir: &Path) -> Result<Output> {
    let start = std::time::Instant::now();
    let output = run_bundle_info(gem_name, working_dir)?;
    if std::env::var_os("RRUBOCOP_DEBUG").is_some() {
        eprintln!(
            "debug: bundle info --path {}: {:.0?}",
            gem_name,
            start.elapsed()
        );
    }
    Ok(output)
}

fn cache_lookup(
    gem_name: &str,
    working_dir: &Path,
    lockfile_mtime: Option<SystemTime>,
) -> Option<PathBuf> {
    let cache = GEM_PATH_CACHE.lock().unwrap();
    let c = cache.as_ref()?;
    if c.working_dir != working_dir || c.lockfile_mtime != lockfile_mtime {
        return None;
    }
    c.entries
        .get(&(working_dir.to_path_buf(), gem_name.to_string()))
        .cloned()
}

fn run_bundle_info(gem_name: &str, working_dir: &Path) -> Result<Output> {
    if needs_mise_exec(working_dir) {
        Command::new("mise")
            .args(["exec", "--", "bundle", "info", "--path", gem_name])
            .current_dir(working_dir)
            .output()
            .with_context(|| {
                format!(
                    "Cannot resolve gem '{}': `mise exec -- bundle` failed. \
                     Ensure mise is installed and `bundle install` has been run.",
                    gem_name
                )
            })
    } else {
        Command::new("bundle")
            .args(["info", "--path", gem_name])
            .current_dir(working_dir)
            .output()
            .with_context(|| {
                format!(
                    "Cannot resolve gem '{}': `bundle` not found on PATH. \
                     Install Bundler or remove inherit_gem/require from your .rubocop.yml.",
                    gem_name
                )
            })
    }
}

fn parse_bundle_path(gem_name: &str, working_dir: &Path, output: &Output) -> Result<PathBuf> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Gem '{}' not found in bundle (working_dir: {}). \
             Run `bundle install` or remove it from inherit_gem. \
             bundle info stderr: {}",
            gem_name,
            working_dir.display(),
            stderr.trim()
        );
    }

    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let path = PathBuf::from(&path_str);
    if !path.exists() {
        anyhow::bail!(
            "Gem '{}' resolved to '{}' but that directory does not exist.",
            gem_name,
            path_str
        );
    }
    Ok(path)
}

fn update_cache(
    gem_name: &str,
    working_dir: &Path,
    lockfile_mtime: Option<SystemTime>,
    path: PathBuf,
) {
    let mut cache = GEM_PATH_CACHE.lock().unwrap();
    let c = cache.get_or_insert_with(|| GemPathCache {
        entries: HashMap::new(),
        lockfile_mtime,
        working_dir: working_dir.to_path_buf(),
    });
    if c.lockfile_mtime != lockfile_mtime || c.working_dir != working_dir {
        c.entries.clear();
        c.lockfile_mtime = lockfile_mtime;
        c.working_dir = working_dir.to_path_buf();
    }
    c.entries
        .insert((working_dir.to_path_buf(), gem_name.to_string()), path);
}

/// Extract all resolved gem paths from the in-process cache.
/// Returns a map of gem_name → gem_root_path.
/// Used by `nitrocop --init` to populate the lockfile.
pub fn drain_resolved_paths() -> HashMap<String, PathBuf> {
    let cache = GEM_PATH_CACHE.lock().unwrap();
    match *cache {
        Some(ref c) => c
            .entries
            .iter()
            .map(|((_, gem_name), path)| (gem_name.clone(), path.clone()))
            .collect(),
        None => HashMap::new(),
    }
}

/// Check if the working directory has a `.ruby-version` or `.tool-versions` file,
/// indicating it may need `mise exec --` to activate the correct Ruby.
/// Only returns true if `mise` is actually available on PATH.
pub(crate) fn needs_mise_exec(working_dir: &Path) -> bool {
    let has_version_file = working_dir.join(".ruby-version").exists()
        || working_dir.join(".tool-versions").exists()
        || working_dir.join(".mise.toml").exists();
    has_version_file && mise_on_path()
}

fn mise_on_path() -> bool {
    static MISE_AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *MISE_AVAILABLE.get_or_init(|| {
        Command::new("mise")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bundle_info_output() {
        // Simulate trimming of bundle info output
        let trimmed = "  /home/user/.gem/ruby/3.2.0/gems/rubocop-shopify-2.15.1  \n".trim();
        assert_eq!(
            trimmed,
            "/home/user/.gem/ruby/3.2.0/gems/rubocop-shopify-2.15.1"
        );
        let path = PathBuf::from(trimmed);
        assert_eq!(
            path.file_name().unwrap().to_str().unwrap(),
            "rubocop-shopify-2.15.1"
        );
    }

    #[test]
    fn cache_key_behavior() {
        // Verify None == None for lockfile mtime comparison
        let a: Option<SystemTime> = None;
        let b: Option<SystemTime> = None;
        assert_eq!(a, b);
    }
}
