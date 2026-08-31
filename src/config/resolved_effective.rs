//! Nested `.rubocop.yml` effective config for a file path.

use std::collections::HashSet;
use std::path::Path;

use crate::cop::EnabledState;

use super::merge::merge_layer_into;
use super::types::{ConfigLayer, InheritMode, NewCopsPolicy};
use super::ResolvedConfig;

impl ResolvedConfig {
    /// Build the effective config for a file under a nested `.rubocop.yml`.
    pub fn effective_config_for_file(&self, file_path: &Path) -> Option<Self> {
        let layer = self.find_dir_layer_for_file(file_path)?;
        let mut effective = self.clone();
        let mut merged = self.layer_snapshot();
        merge_layer_into(&mut merged, layer, Some(&layer.inherit_mode));
        self.apply_merged_layer(&mut effective, &merged);
        self.apply_project_mentions(&mut effective, layer);
        normalize_disabled_by_default(&mut effective);
        Some(effective)
    }

    /// Find the nearest directory-specific config layer, if any.
    fn find_dir_layer_for_file(&self, file_path: &Path) -> Option<&ConfigLayer> {
        if self.dir_overrides.is_empty() {
            return None;
        }
        for (dir, layer) in &self.dir_overrides {
            if file_path.starts_with(dir) {
                return Some(layer);
            }
        }
        self.find_dir_layer_relativized(file_path)
    }

    fn find_dir_layer_relativized(&self, file_path: &Path) -> Option<&ConfigLayer> {
        let config_dir = self.config_dir.as_ref()?;
        let rel_path = file_path.strip_prefix(config_dir).ok()?;
        for (dir, layer) in &self.dir_overrides {
            if let Ok(rel_dir) = dir.strip_prefix(config_dir) {
                if rel_path.starts_with(rel_dir) {
                    return Some(layer);
                }
            }
        }
        None
    }

    fn layer_snapshot(&self) -> ConfigLayer {
        ConfigLayer {
            cop_configs: self.cop_configs.clone(),
            department_configs: self.department_configs.clone(),
            global_excludes: self.global_excludes.clone(),
            new_cops: Some(match self.new_cops {
                NewCopsPolicy::Enable => "enable".to_string(),
                NewCopsPolicy::Disable => "disable".to_string(),
            }),
            disabled_by_default: Some(self.disabled_by_default),
            inherit_mode: InheritMode::default(),
            require_enabled_cops: HashSet::new(),
            require_enabled_depts: HashSet::new(),
            require_known_cops: self.require_known_cops.clone(),
            require_departments: self.require_departments.clone(),
            user_mentioned_cops: self.project_mentioned_cops.clone(),
            user_mentioned_depts: self.project_mentioned_depts.clone(),
            target_ruby_version: self.target_ruby_version,
            target_rails_version: self.target_rails_version,
            active_support_extensions_enabled: Some(self.active_support_extensions_enabled),
            migrated_schema_version: self.migrated_schema_version.clone(),
        }
    }

    fn apply_merged_layer(&self, effective: &mut Self, merged: &ConfigLayer) {
        effective.cop_configs = merged.cop_configs.clone();
        effective.department_configs = merged.department_configs.clone();
        effective.global_excludes = merged.global_excludes.clone();
        effective.new_cops = match merged.new_cops.as_deref() {
            Some("enable") => NewCopsPolicy::Enable,
            _ => NewCopsPolicy::Disable,
        };
        effective.disabled_by_default = merged
            .disabled_by_default
            .unwrap_or(self.disabled_by_default);
        effective.require_known_cops = merged.require_known_cops.clone();
        effective.require_departments = merged.require_departments.clone();
        effective.target_ruby_version = merged.target_ruby_version;
        effective.target_rails_version = merged.target_rails_version;
        effective.active_support_extensions_enabled = merged
            .active_support_extensions_enabled
            .unwrap_or(self.active_support_extensions_enabled);
        effective.migrated_schema_version = merged.migrated_schema_version.clone();
    }

    fn apply_project_mentions(&self, effective: &mut Self, layer: &ConfigLayer) {
        effective
            .project_mentioned_cops
            .extend(layer.cop_configs.keys().cloned());
        effective
            .project_mentioned_depts
            .extend(layer.department_configs.keys().cloned());
        for (dept_name, dept_cfg) in &layer.department_configs {
            update_enabled_dept(&mut effective.project_enabled_depts, dept_name, dept_cfg.enabled);
        }
    }
}

fn update_enabled_dept(
    depts: &mut HashSet<String>,
    dept_name: &str,
    enabled: EnabledState,
) {
    if enabled == EnabledState::True {
        depts.insert(dept_name.to_string());
    } else if enabled != EnabledState::Unset {
        depts.remove(dept_name);
    }
}

fn normalize_disabled_by_default(effective: &mut ResolvedConfig) {
    if !effective.disabled_by_default {
        return;
    }
    for (cop_name, cop_cfg) in &mut effective.cop_configs {
        if cop_cfg.enabled == EnabledState::True
            && !effective.project_mentioned_cops.contains(cop_name)
        {
            cop_cfg.enabled = EnabledState::Unset;
        }
    }
    for (dept_name, dept_cfg) in &mut effective.department_configs {
        if dept_cfg.enabled == EnabledState::True
            && !effective.project_mentioned_depts.contains(dept_name)
        {
            dept_cfg.enabled = EnabledState::Unset;
        }
    }
}
