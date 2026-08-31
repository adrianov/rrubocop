//! Per-cop config merging.

use std::collections::HashSet;

use serde_yml::Value;

use crate::cop::{CopConfig, EnabledState};

use super::merge::{merge_exclude_list, merge_include_list};
use super::types::InheritMode;

/// Merge a single cop's overlay config into its base config.
pub(crate) fn merge_cop_config(
    base: &mut CopConfig,
    overlay: &CopConfig,
    inherit_mode: Option<&InheritMode>,
) {
    merge_cop_scalars(base, overlay);
    merge_cop_lists(base, overlay, inherit_mode);
    merge_cop_options(base, overlay);
}

fn merge_cop_scalars(base: &mut CopConfig, overlay: &CopConfig) {
    if overlay.enabled != EnabledState::Unset {
        base.enabled = overlay.enabled;
    }
    if overlay.severity.is_some() {
        base.severity = overlay.severity;
    }
}

fn merge_cop_lists(
    base: &mut CopConfig,
    overlay: &CopConfig,
    inherit_mode: Option<&InheritMode>,
) {
    let should_merge_include = inherit_mode
        .map(|im| im.merge.contains("Include"))
        .unwrap_or(false);
    let should_override_exclude = inherit_mode
        .map(|im| im.override_keys.contains("Exclude"))
        .unwrap_or(false);
    merge_exclude_list(&mut base.exclude, &overlay.exclude, should_override_exclude);
    merge_include_list(&mut base.include, &overlay.include, should_merge_include);
}

fn merge_cop_options(base: &mut CopConfig, overlay: &CopConfig) {
    let cop_inherit_mode = cop_inherit_merge_keys(overlay);
    for (key, value) in &overlay.options {
        if key == "inherit_mode" {
            continue;
        }
        merge_one_option(base, key, value, &cop_inherit_mode);
    }
}

fn cop_inherit_merge_keys(overlay: &CopConfig) -> HashSet<String> {
    let Some(mode) = overlay.options.get("inherit_mode") else {
        return HashSet::new();
    };
    let Some(map) = mode.as_mapping() else {
        return HashSet::new();
    };
    seq_to_string_set(map.get(Value::String("merge".to_string())))
}

fn seq_to_string_set(value: Option<&Value>) -> HashSet<String> {
    value
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn merge_one_option(
    base: &mut CopConfig,
    key: &str,
    value: &Value,
    cop_inherit_mode: &HashSet<String>,
) {
    if let (Some(Value::Mapping(base_map)), Value::Mapping(overlay_map)) =
        (base.options.get(key), value)
    {
        let mut merged = base_map.clone();
        for (k, v) in overlay_map {
            merged.insert(k.clone(), v.clone());
        }
        base.options.insert(key.to_string(), Value::Mapping(merged));
    } else if cop_inherit_mode.contains(key) {
        merge_inherit_array_option(base, key, value);
    } else {
        base.options.insert(key.to_string(), value.clone());
    }
}

fn merge_inherit_array_option(base: &mut CopConfig, key: &str, value: &Value) {
    if let (Some(Value::Sequence(base_seq)), Value::Sequence(overlay_seq)) =
        (base.options.get(key), value)
    {
        let mut merged = base_seq.clone();
        for item in overlay_seq {
            if !merged.contains(item) {
                merged.push(item.clone());
            }
        }
        base.options
            .insert(key.to_string(), Value::Sequence(merged));
    } else {
        base.options.insert(key.to_string(), value.clone());
    }
}
