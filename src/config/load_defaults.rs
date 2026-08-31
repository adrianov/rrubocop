//! RuboCop gem `config/default.yml` as the lowest-priority base layer.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_yml::Value;

use super::gem_path;
use super::parse::parse_config_layer;
use super::types::ConfigLayer;

/// Minimal AllCops.Exclude patterns matching rubocop's config/default.yml.
pub(crate) fn fallback_default_excludes() -> ConfigLayer {
    let mut layer = ConfigLayer::empty();
    layer.global_excludes = vec![
        "node_modules/**/*".to_string(),
        "tmp/**/*".to_string(),
        "vendor/**/*".to_string(),
        ".git/**/*".to_string(),
    ];
    layer
}

fn rubocop_gem_root(
    working_dir: &Path,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> Option<PathBuf> {
    if let Some(path) = gem_cache.and_then(|c| c.get("rubocop")) {
        return Some(path.clone());
    }
    gem_path::resolve_gem_path("rubocop", working_dir).ok()
}

fn read_default_yml(gem_root: &Path, working_dir: &Path) -> Option<(PathBuf, String)> {
    let default_config = gem_root.join("config").join("default.yml");
    if !default_config.exists() {
        return None;
    }
    let contents = super::yaml_read::load_yaml_text(&default_config, working_dir).ok()?;
    Some((default_config, contents))
}

fn parse_default_yml(path: &Path, contents: &str) -> Option<Value> {
    match serde_yml::from_str(contents) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!(
                "warning: failed to parse rubocop default config {}: {e}",
                path.display()
            );
            None
        }
    }
}

fn collect_known_cops(raw: &Value) -> HashSet<String> {
    let Value::Mapping(map) = raw else {
        return HashSet::new();
    };
    map.keys()
        .filter_map(|k| k.as_str())
        .filter(|k| k.contains('/'))
        .map(|k| k.to_string())
        .collect()
}

/// Load rubocop's `config/default.yml` as the base layer, or fallback excludes.
///
/// Also returns known cop names from the installed gem for version awareness.
pub(crate) fn try_load_rubocop_defaults(
    working_dir: &Path,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> (ConfigLayer, HashSet<String>) {
    let Some(gem_root) = rubocop_gem_root(working_dir, gem_cache) else {
        return (fallback_default_excludes(), HashSet::new());
    };
    let Some((path, contents)) = read_default_yml(&gem_root, working_dir) else {
        return (fallback_default_excludes(), HashSet::new());
    };
    let Some(raw) = parse_default_yml(&path, &contents) else {
        return (fallback_default_excludes(), HashSet::new());
    };
    let known_cops = collect_known_cops(&raw);
    (parse_config_layer(&raw), known_cops)
}
