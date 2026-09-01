//! Resolve vendored gem config YAML from Gemfile.lock + embedded resources.
//!
//! Does not call `bundle info` or write gem trees to disk. Version comes from
//! `Gemfile.lock` / `gems.locked`; YAML is compiled into the binary.

use std::path::Path;

use anyhow::{Context, Result};

use super::gem_configs;
use super::gem_path_version::select_version;

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

/// Stable virtual path for visited-set / error messages (not on disk).
pub fn virtual_config_path(gem_name: &str, version: &str, rel_path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!(
        "/__rrubocop_gem__/{gem_name}/{version}/{rel_path}"
    ))
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
