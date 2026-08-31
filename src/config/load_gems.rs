//! `inherit_gem` config loading.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_yml::Value;

use super::gem_path;
use super::load_recursive::load_config_recursive;
use super::load_require::collect_yaml_strings;
use super::merge::merge_layer_into;
use super::types::ConfigLayer;

fn resolve_gem_root(
    gem_name: &str,
    working_dir: &Path,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> Result<PathBuf> {
    if let Some(path) = gem_cache.and_then(|c| c.get(gem_name)) {
        return Ok(path.clone());
    }
    gem_path::resolve_gem_path(gem_name, working_dir).with_context(|| {
        format!(
            "inherit_gem: failed to resolve gem '{gem_name}'. \
             Run `bundle install` to install it, or remove it from inherit_gem in .rubocop.yml."
        )
    })
}

fn load_inherit_gem_file(
    gem_name: &str,
    full_path: &Path,
    working_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> Result<ConfigLayer> {
    // See WARNING in load_require — do NOT make excludes absolute to gem dir.
    load_config_recursive(full_path, working_dir, visited, gem_cache).with_context(|| {
        format!(
            "inherit_gem: failed to load config {} from gem '{gem_name}'",
            full_path.display()
        )
    })
}

/// Resolve `inherit_gem` entries to config layers (hard-fail if gem missing).
pub(crate) fn resolve_inherit_gem(
    gem_name: &str,
    paths_value: &Value,
    working_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> Result<Vec<ConfigLayer>> {
    let gem_root = resolve_gem_root(gem_name, working_dir, gem_cache)?;
    let mut layers = Vec::new();
    for rel_path in &collect_yaml_strings(paths_value) {
        let full_path = gem_root.join(rel_path);
        if !full_path.exists() {
            anyhow::bail!(
                "inherit_gem: config file not found: {} (gem '{gem_name}')",
                full_path.display(),
            );
        }
        layers.push(load_inherit_gem_file(
            gem_name,
            &full_path,
            working_dir,
            visited,
            gem_cache,
        )?);
    }
    Ok(layers)
}

fn merge_inherit_layers(base_layer: &mut ConfigLayer, layers: Vec<ConfigLayer>) {
    for layer in layers {
        base_layer
            .user_mentioned_cops
            .extend(layer.user_mentioned_cops.iter().cloned());
        base_layer
            .user_mentioned_depts
            .extend(layer.user_mentioned_depts.iter().cloned());
        merge_layer_into(base_layer, &layer, None);
    }
}

/// Process `inherit_gem:` map entries into `base_layer`.
pub(crate) fn process_inherit_gem(
    map: &serde_yml::Mapping,
    working_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    gem_cache: Option<&HashMap<String, PathBuf>>,
    base_layer: &mut ConfigLayer,
) {
    let Some(Value::Mapping(gem_map)) = map.get(Value::String("inherit_gem".to_string())) else {
        return;
    };
    for (gem_key, gem_paths) in gem_map {
        let Some(gem_name) = gem_key.as_str() else {
            continue;
        };
        match resolve_inherit_gem(gem_name, gem_paths, working_dir, visited, gem_cache) {
            Ok(layers) => merge_inherit_layers(base_layer, layers),
            Err(e) => eprintln!("warning: {e:#}"),
        }
    }
}
