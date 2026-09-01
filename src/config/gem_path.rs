//! Resolve gem config YAML: vendored embed first, then local `bundle info`.
//!
//! Public plugin defaults are compiled into the binary (version from
//! `Gemfile.lock` / `gems.locked`). Private or unlisted gems fall back to
//! `bundle info --path` so `inherit_gem` / `require` still work without
//! vendoring proprietary YAML.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::gem_configs;
use super::gem_path_local::resolve_local_root;
use super::gem_path_version::select_version;

/// Where to read a gem's RuboCop YAML from.
pub enum GemConfigSrc {
    /// Installed gem root (explicit cache or `bundle info --path`).
    Disk(PathBuf),
    /// Vendored bytes + version (virtual path via [`virtual_config_path`]).
    Embed { version: String, yaml: &'static str },
}

/// Prefer `gem_cache`, then vendored embed, then local `bundle info --path`.
pub fn resolve_gem_config(
    gem_name: &str,
    rel_path: &str,
    working_dir: &Path,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> Result<GemConfigSrc> {
    if let Some(root) = gem_cache.and_then(|c| c.get(gem_name)) {
        return Ok(GemConfigSrc::Disk(root.clone()));
    }
    if has_vendored(gem_name) {
        let (version, yaml) = embedded_yaml(gem_name, rel_path, working_dir)?;
        return Ok(GemConfigSrc::Embed { version, yaml });
    }
    Ok(GemConfigSrc::Disk(resolve_local_root(gem_name, working_dir)?))
}

/// Select lockfile/baseline version and return embedded YAML for `rel_path`.
pub fn embedded_yaml(
    gem_name: &str,
    rel_path: &str,
    working_dir: &Path,
) -> Result<(String, &'static str)> {
    let version = select_version(gem_name, working_dir)?;
    let yaml = gem_configs::file(gem_name, &version, rel_path).with_context(|| {
        format!(
            "no vendored config {gem_name}@{version}/{rel_path}; \
             add it to src/resources/gem_configs_manifest.json and re-run \
             scripts/fetch_gem_configs.py"
        )
    })?;
    Ok((version, yaml))
}

fn has_vendored(gem_name: &str) -> bool {
    !gem_configs::versions_for(gem_name).is_empty()
}

/// Stable virtual path for visited-set / error messages (not on disk).
pub fn virtual_config_path(gem_name: &str, version: &str, rel_path: &str) -> PathBuf {
    PathBuf::from(format!("/__rrubocop_gem__/{gem_name}/{version}/{rel_path}"))
}

/// Whether `working_dir` likely needs `mise exec` for Ruby subprocesses (ERB).
pub(crate) fn needs_mise_exec(working_dir: &Path) -> bool {
    let has_version_file = working_dir.join(".ruby-version").exists()
        || working_dir.join(".tool-versions").exists()
        || working_dir.join(".mise.toml").exists();
    has_version_file && mise_on_path()
}

fn mise_on_path() -> bool {
    static MISE_AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *MISE_AVAILABLE.get_or_init(|| {
        std::process::Command::new("mise")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}
