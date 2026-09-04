//! Config file discovery and nested directory overrides.

use std::path::{Path, PathBuf};

use serde_yml::Value;

use super::parse::parse_config_layer;
use super::types::ConfigLayer;

fn push_sub_config_dir(dirs: &mut Vec<PathBuf>, root: &Path, entry: &ignore::DirEntry) {
    if !entry.file_type().is_some_and(|ft| ft.is_file()) {
        return;
    }
    if entry.file_name() != ".rubocop.yml" {
        return;
    }
    let Some(parent) = entry.path().parent() else {
        return;
    };
    if parent != root {
        dirs.push(parent.to_path_buf());
    }
}

/// Walk the project tree for nested `.rubocop.yml` dirs (deepest-first).
pub(crate) fn discover_sub_config_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();
    for entry in walker.flatten() {
        push_sub_config_dir(&mut dirs, root, &entry);
    }
    dirs.sort_by_key(|b| std::cmp::Reverse(b.as_os_str().len()));
    dirs
}

fn layer_has_effect(layer: &ConfigLayer) -> bool {
    !layer.cop_configs.is_empty()
        || !layer.department_configs.is_empty()
        || !layer.global_excludes.is_empty()
        || layer.new_cops.is_some()
        || layer.disabled_by_default.is_some()
        || layer.target_ruby_version.is_some()
        || layer.target_rails_version.is_some()
        || layer.active_support_extensions_enabled.is_some()
        || layer.migrated_schema_version.is_some()
}

fn try_parse_dir_override(dir: &Path, root: &Path) -> Option<ConfigLayer> {
    let config_path = dir.join(".rubocop.yml");
    let contents = super::yaml_read::load_yaml_text(&config_path, root).ok()?;
    let raw: Value = match serde_yml::from_str(&contents) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "warning: failed to parse nested config {}: {e}",
                config_path.display()
            );
            return None;
        }
    };
    Some(parse_config_layer(&raw))
}

/// Load per-directory config layers from nested `.rubocop.yml` files.
pub(crate) fn load_dir_overrides(root: &Path) -> Vec<(PathBuf, ConfigLayer)> {
    let mut overrides = Vec::new();
    for dir in discover_sub_config_dirs(root) {
        let Some(layer) = try_parse_dir_override(&dir, root) else {
            continue;
        };
        if layer_has_effect(&layer) {
            overrides.push((dir, layer));
        }
    }
    overrides
}

fn walk_up_once(start: &Path, filename: &str) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join(filename);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Walk up from `start_dir` looking for a config file name.
pub(crate) fn walk_up_for(start_dir: &Path, filename: &str) -> Option<PathBuf> {
    if let Some(found) = walk_up_once(start_dir, filename) {
        return Some(found);
    }
    let canonical = std::fs::canonicalize(start_dir).ok()?;
    if canonical == start_dir {
        return None;
    }
    walk_up_once(&canonical, filename)
}

/// Prefer `.rubocop.yml`, else `.standard.yml`, else user home / XDG (RuboCop).
pub(crate) fn find_config(start_dir: &Path) -> Option<PathBuf> {
    walk_up_for(start_dir, ".rubocop.yml")
        .or_else(|| walk_up_for(start_dir, ".standard.yml"))
        .or_else(find_user_dotfile)
        .or_else(find_user_xdg_config)
}

fn find_user_dotfile() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let file = PathBuf::from(home).join(".rubocop.yml");
    file.is_file().then_some(file)
}

fn find_user_xdg_config() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
        })?;
    let file = base.join("rubocop").join("config.yml");
    file.is_file().then_some(file)
}
