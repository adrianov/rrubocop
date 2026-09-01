//! Pre-compiled CopFilterSet construction.

use std::collections::HashMap;

use globset::GlobSet;
use regex::RegexSet;

use crate::cop::CopRegistry;

use super::discover::discover_sub_config_dirs;
use super::filter::{CopFilter, CopFilterSet};
use super::globutil::{build_glob_set, build_regex_set, extract_ruby_regexp};
use super::resolved_state::{
    effective_exclude, effective_include, lint_constant_resolution_on,
    redundant_constant_base_blocked, resolve_enabled_state, state_to_enabled,
    version_gate_disables,
};
use super::ResolvedConfig;

impl ResolvedConfig {
    /// Build pre-compiled cop filters for fast per-file enablement checks.
    pub fn build_cop_filters(&self, registry: &CopRegistry) -> CopFilterSet {
        let (global_exclude, global_exclude_patterns, global_exclude_re) =
            self.build_global_excludes();
        let filters = self.build_all_filters(registry);
        let (universal_cop_indices, pattern_cop_indices) = partition_filter_indices(&filters);
        CopFilterSet {
            global_exclude,
            global_exclude_patterns,
            global_exclude_re,
            filters,
            name_to_index: name_index_map(registry),
            config_dir: self.config_dir.clone(),
            base_dir: self.base_dir.clone(),
            scan_root: None,
            sub_config_dirs: self.sub_config_dirs(),
            universal_cop_indices,
            pattern_cop_indices,
            migrated_schema_version: self.migrated_schema_version.clone(),
        }
    }

    /// Filters for file discovery (`--list-target-files`): AllCops.Exclude only.
    pub fn build_discover_filters(&self) -> CopFilterSet {
        let (global_exclude, global_exclude_patterns, global_exclude_re) =
            self.build_global_excludes();
        CopFilterSet {
            global_exclude,
            global_exclude_patterns,
            global_exclude_re,
            filters: Vec::new(),
            name_to_index: HashMap::new(),
            config_dir: self.config_dir.clone(),
            base_dir: self.base_dir.clone(),
            scan_root: None,
            sub_config_dirs: self.sub_config_dirs(),
            universal_cop_indices: Vec::new(),
            pattern_cop_indices: Vec::new(),
            migrated_schema_version: self.migrated_schema_version.clone(),
        }
    }

    fn build_all_filters(&self, registry: &CopRegistry) -> Vec<CopFilter> {
        let lcr_on = lint_constant_resolution_on(self);
        registry
            .cops()
            .iter()
            .map(|cop| {
                self.filter_for_cop(
                    cop.name(),
                    cop.default_enabled(),
                    cop.default_include(),
                    cop.default_exclude(),
                    lcr_on,
                )
            })
            .collect()
    }

    fn build_global_excludes(&self) -> (GlobSet, Vec<String>, Option<RegexSet>) {
        let pats: Vec<&str> = self.global_excludes.iter().map(|s| s.as_str()).collect();
        let global_exclude = build_glob_set(&pats).unwrap_or_else(GlobSet::empty);
        let global_exclude_patterns = self
            .global_excludes
            .iter()
            .filter(|pattern| extract_ruby_regexp(pattern).is_none())
            .cloned()
            .collect();
        let global_exclude_re = build_regex_set(&pats);
        (global_exclude, global_exclude_patterns, global_exclude_re)
    }

    fn sub_config_dirs(&self) -> Vec<std::path::PathBuf> {
        if self.dir_overrides.is_empty() {
            self.config_dir
                .as_ref()
                .map(|cd| discover_sub_config_dirs(cd))
                .unwrap_or_default()
        } else {
            self.dir_overrides
                .iter()
                .map(|(dir, _)| dir.clone())
                .collect()
        }
    }

    fn filter_for_cop(
        &self,
        name: &str,
        default_enabled: bool,
        default_include: &[&str],
        default_exclude: &[&str],
        lcr_on: bool,
    ) -> CopFilter {
        if !self.cop_filter_enabled(name, default_enabled, lcr_on) {
            return disabled_filter();
        }
        self.enabled_filter(name, default_include, default_exclude)
    }

    fn cop_filter_enabled(&self, name: &str, default_enabled: bool, lcr_on: bool) -> bool {
        let config = self.cop_configs.get(name);
        let dept = name.split('/').next().unwrap_or("");
        let inputs = self.enable_inputs(name, dept, config, default_enabled);
        let state = resolve_enabled_state(&inputs);
        let mut enabled = state_to_enabled(state, self.new_cops, self.disabled_by_default, default_enabled);
        if enabled && version_gate_disables(self, name, dept, config, false) {
            enabled = false;
        }
        if enabled && redundant_constant_base_blocked(name, lcr_on) {
            enabled = false;
        }
        enabled
    }

    fn enabled_filter(
        &self,
        name: &str,
        default_include: &[&str],
        default_exclude: &[&str],
    ) -> CopFilter {
        let config = self.cop_configs.get(name);
        let dept = name.split('/').next().unwrap_or("");
        let dept_config = self.department_configs.get(dept);
        let include_patterns = effective_include(config, dept_config, default_include);
        let exclude_patterns = effective_exclude(config, dept_config, default_exclude);
        CopFilter {
            enabled: true,
            include_set: build_glob_set(&include_patterns),
            exclude_set: build_glob_set(&exclude_patterns),
            include_re: build_regex_set(&include_patterns),
            exclude_re: build_regex_set(&exclude_patterns),
        }
    }
}

fn disabled_filter() -> CopFilter {
    CopFilter {
        enabled: false,
        include_set: None,
        exclude_set: None,
        include_re: None,
        exclude_re: None,
    }
}

fn partition_filter_indices(filters: &[CopFilter]) -> (Vec<usize>, Vec<usize>) {
    let mut universal = Vec::new();
    let mut pattern = Vec::new();
    for (i, filter) in filters.iter().enumerate() {
        if filter.is_universal() {
            universal.push(i);
        } else if filter.enabled {
            pattern.push(i);
        }
    }
    (universal, pattern)
}

fn name_index_map(registry: &CopRegistry) -> std::collections::HashMap<String, usize> {
    registry
        .cops()
        .iter()
        .enumerate()
        .map(|(i, cop)| (cop.name().to_string(), i))
        .collect()
}
