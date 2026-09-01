//! Gemfile.lock / gems.locked version metadata for config resolution.

use std::path::{Path, PathBuf};

use super::discover::load_dir_overrides;
use super::load_defaults::fallback_default_excludes;
use super::resolved::ResolvedConfig;
use super::ruby_ver::parse_gem_version_from_lockfile;
use super::types::ConfigLayer;

pub(crate) fn lockfile_gem_version(lockfile_dir: &Path, gem_name: &str) -> Option<f64> {
    for lock_name in &["Gemfile.lock", "gems.locked"] {
        let lock_path = lockfile_dir.join(lock_name);
        if let Ok(content) = std::fs::read_to_string(&lock_path) {
            if let Some(ver) = parse_gem_version_from_lockfile(&content, gem_name) {
                return Some(ver);
            }
        }
    }
    None
}

pub(crate) struct LockfileMeta {
    pub target_rails_version: Option<f64>,
    pub railties_in_lockfile: bool,
    pub railties_version: Option<f64>,
    pub rack_version: Option<f64>,
}

fn railties_from_lock(lockfile_dir: &Path) -> Option<f64> {
    lockfile_gem_version(lockfile_dir, "railties")
}

pub(crate) fn resolve_lockfile_meta(base: &ConfigLayer, lockfile_dir: &Path) -> LockfileMeta {
    let mut railties_version = None;
    let mut railties_in_lockfile = false;
    let target_rails_version = base.target_rails_version.or_else(|| {
        let ver = railties_from_lock(lockfile_dir)?;
        railties_in_lockfile = true;
        railties_version = Some(ver);
        Some(ver)
    });
    if !railties_in_lockfile && base.target_rails_version.is_some() {
        if let Some(ver) = railties_from_lock(lockfile_dir) {
            railties_in_lockfile = true;
            railties_version = Some(ver);
        }
    }
    LockfileMeta {
        target_rails_version,
        railties_in_lockfile,
        railties_version,
        rack_version: lockfile_gem_version(lockfile_dir, "rack"),
    }
}

pub(crate) fn empty_resolved_no_config(config_dir: PathBuf) -> ResolvedConfig {
    let base_dir = std::env::current_dir().unwrap_or_else(|_| config_dir.clone());
    let defaults = fallback_default_excludes();
    let railties_version = lockfile_gem_version(&base_dir, "railties");
    let rack_version = lockfile_gem_version(&base_dir, "rack");
    ResolvedConfig {
        config_dir: Some(config_dir.clone()),
        scan_root: Some(config_dir.clone()),
        dir_overrides: load_dir_overrides(&config_dir),
        base_dir: Some(base_dir),
        global_excludes: defaults.global_excludes,
        target_rails_version: railties_version,
        railties_in_lockfile: railties_version.is_some(),
        railties_version,
        rack_version,
        ..ResolvedConfig::empty()
    }
}
