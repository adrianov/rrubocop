//! `inherit_gem` config loading.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_yml::Value;

use super::gem_path;
use super::load_recursive::{load_config_recursive, load_config_recursive_inner};
use super::load_require::collect_yaml_strings;
use super::merge::merge_layer_into;
use super::types::ConfigLayer;

fn load_inherit_gem_yaml(
    gem_name: &str,
    rel_path: &str,
    working_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> Result<ConfigLayer> {
    if let Some(root) = gem_cache.and_then(|c| c.get(gem_name)) {
        let full_path = root.join(rel_path);
        return load_config_recursive(&full_path, working_dir, visited, gem_cache).with_context(
            || {
                format!(
                    "inherit_gem: failed to load config {} from gem '{gem_name}'",
                    full_path.display()
                )
            },
        );
    }
    let (version, yaml) = gem_path::embedded_yaml(gem_name, rel_path, working_dir).with_context(
        || {
            format!(
                "inherit_gem: failed to resolve vendored config for gem '{gem_name}'. \
                 Ensure it is listed in Gemfile.lock and vendored in rrubocop \
                 (src/resources/gem_configs_manifest.json)."
            )
        },
    )?;
    let path = gem_path::virtual_config_path(gem_name, &version, rel_path);
    // Do NOT make excludes absolute to a gem dir — patterns stay project-relative.
    load_config_recursive_inner(&path, working_dir, visited, gem_cache, Some(yaml)).with_context(
        || format!("inherit_gem: failed to load config {rel_path} from gem '{gem_name}'"),
    )
}

/// Resolve `inherit_gem` entries to config layers.
pub(crate) fn resolve_inherit_gem(
    gem_name: &str,
    paths_value: &Value,
    working_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    gem_cache: Option<&HashMap<String, PathBuf>>,
) -> Result<Vec<ConfigLayer>> {
    let mut layers = Vec::new();
    for rel_path in &collect_yaml_strings(paths_value) {
        layers.push(load_inherit_gem_yaml(
            gem_name,
            rel_path,
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
