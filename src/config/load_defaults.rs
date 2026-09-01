//! RuboCop gem `config/default.yml` as the lowest-priority base layer.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_yml::Value;

use super::gem_path;
use super::parse::parse_config_layer;
use super::types::ConfigLayer;
use super::yaml_read;

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

fn defaults_from_cache(
    working_dir: &Path,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> Option<(String, String)> {
    let root = gem_cache?.get("rubocop")?;
    let path = root.join("config").join("default.yml");
    let contents = yaml_read::load_yaml_text(&path, working_dir).ok()?;
    Some((path.display().to_string(), contents))
}

fn defaults_from_embed(working_dir: &Path) -> Option<(String, String)> {
    let (_ver, yaml) = gem_path::embedded_yaml("rubocop", "config/default.yml", working_dir).ok()?;
    Some(("rubocop/config/default.yml".into(), yaml.to_string()))
}

fn parse_default_yml(label: &str, contents: &str) -> Option<Value> {
    match serde_yml::from_str(contents) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("warning: failed to parse rubocop default config {label}: {e}");
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
/// Also returns known cop names from the vendored/default gem for version awareness.
pub(crate) fn try_load_rubocop_defaults(
    working_dir: &Path,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> (ConfigLayer, HashSet<String>) {
    let Some((label, contents)) =
        defaults_from_cache(working_dir, gem_cache).or_else(|| defaults_from_embed(working_dir))
    else {
        return (fallback_default_excludes(), HashSet::new());
    };
    let Some(raw) = parse_default_yml(&label, &contents) else {
        return (fallback_default_excludes(), HashSet::new());
    };
    let known_cops = collect_known_cops(&raw);
    (parse_config_layer(&raw), known_cops)
}
