//! Layer / department config merging.

use crate::cop::EnabledState;

use super::merge_cop::merge_cop_config;
use super::types::{ConfigLayer, DepartmentConfig, InheritMode};

pub(crate) fn merge_layer_into(
    base: &mut ConfigLayer,
    overlay: &ConfigLayer,
    inherit_mode: Option<&InheritMode>,
) {
    merge_global_excludes(base, overlay, inherit_mode);
    merge_layer_scalars(base, overlay);
    merge_department_map(base, overlay, inherit_mode);
    merge_cop_map(base, overlay, inherit_mode);
    merge_require_sets(base, overlay);
}

fn merge_global_excludes(
    base: &mut ConfigLayer,
    overlay: &ConfigLayer,
    inherit_mode: Option<&InheritMode>,
) {
    // RuboCop ALWAYS merges AllCops.Exclude (union), unlike per-cop Exclude
    // which replaces by default. The only exception is when inherit_mode
    // explicitly sets `override: [Exclude]`.
    if overlay.global_excludes.is_empty() {
        return;
    }
    let should_replace = match inherit_mode {
        None => false,
        Some(mode) => mode.override_keys.contains("Exclude"),
    };
    merge_exclude_list(
        &mut base.global_excludes,
        &overlay.global_excludes,
        should_replace,
    );
}

fn merge_layer_scalars(base: &mut ConfigLayer, overlay: &ConfigLayer) {
    // NewCops / DisabledByDefault / version fields: last writer wins
    if overlay.new_cops.is_some() {
        base.new_cops.clone_from(&overlay.new_cops);
    }
    if overlay.disabled_by_default.is_some() {
        base.disabled_by_default = overlay.disabled_by_default;
    }
    if overlay.target_ruby_version.is_some() {
        base.target_ruby_version = overlay.target_ruby_version;
    }
    if overlay.target_rails_version.is_some() {
        base.target_rails_version = overlay.target_rails_version;
    }
    if overlay.active_support_extensions_enabled.is_some() {
        base.active_support_extensions_enabled = overlay.active_support_extensions_enabled;
    }
    if overlay.migrated_schema_version.is_some() {
        base.migrated_schema_version
            .clone_from(&overlay.migrated_schema_version);
    }
}

fn merge_department_map(
    base: &mut ConfigLayer,
    overlay: &ConfigLayer,
    inherit_mode: Option<&InheritMode>,
) {
    for (dept_name, overlay_dept) in &overlay.department_configs {
        match base.department_configs.get_mut(dept_name) {
            Some(base_dept) => {
                merge_department_config(base_dept, overlay_dept, inherit_mode);
            }
            None => {
                base.department_configs
                    .insert(dept_name.clone(), overlay_dept.clone());
            }
        }
    }
}

fn merge_cop_map(
    base: &mut ConfigLayer,
    overlay: &ConfigLayer,
    inherit_mode: Option<&InheritMode>,
) {
    for (cop_name, overlay_config) in &overlay.cop_configs {
        match base.cop_configs.get_mut(cop_name) {
            Some(base_config) => {
                merge_cop_config(base_config, overlay_config, inherit_mode);
            }
            None => {
                base.cop_configs
                    .insert(cop_name.clone(), overlay_config.clone());
            }
        }
        // Track require-originated enabled state through merges.
        if overlay.require_enabled_cops.contains(cop_name) {
            base.require_enabled_cops.insert(cop_name.clone());
        } else if overlay_config.enabled != EnabledState::Unset {
            base.require_enabled_cops.remove(cop_name);
        }
    }
}

fn merge_require_sets(base: &mut ConfigLayer, overlay: &ConfigLayer) {
    for (dept_name, overlay_dept) in &overlay.department_configs {
        if overlay.require_enabled_depts.contains(dept_name) {
            base.require_enabled_depts.insert(dept_name.clone());
        } else if overlay_dept.enabled != EnabledState::Unset {
            base.require_enabled_depts.remove(dept_name);
        }
    }
    // Propagate require-known cops and departments (union — once known, always known)
    for cop in &overlay.require_known_cops {
        base.require_known_cops.insert(cop.clone());
    }
    for dept in &overlay.require_departments {
        base.require_departments.insert(dept.clone());
    }
}

/// Merge a single department's overlay config into its base config.
pub(crate) fn merge_department_config(
    base: &mut DepartmentConfig,
    overlay: &DepartmentConfig,
    inherit_mode: Option<&InheritMode>,
) {
    if overlay.enabled != EnabledState::Unset {
        base.enabled = overlay.enabled;
    }
    let should_merge_include = inherit_mode
        .map(|im| im.merge.contains("Include"))
        .unwrap_or(false);
    let should_override_exclude = inherit_mode
        .map(|im| im.override_keys.contains("Exclude"))
        .unwrap_or(false);
    merge_exclude_list(&mut base.exclude, &overlay.exclude, should_override_exclude);
    merge_include_list(&mut base.include, &overlay.include, should_merge_include);
}

pub(crate) fn merge_exclude_list(
    base: &mut Vec<String>,
    overlay: &[String],
    should_override: bool,
) {
    if should_override {
        if !overlay.is_empty() {
            *base = overlay.to_vec();
        }
    } else {
        for exc in overlay {
            if !base.contains(exc) {
                base.push(exc.clone());
            }
        }
    }
}

pub(crate) fn merge_include_list(base: &mut Vec<String>, overlay: &[String], should_merge: bool) {
    if overlay.is_empty() {
        return;
    }
    if should_merge {
        for inc in overlay {
            if !base.contains(inc) {
                base.push(inc.clone());
            }
        }
    } else {
        *base = overlay.to_vec();
    }
}
