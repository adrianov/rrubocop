//! Shared enabled-state and Include/Exclude helpers for ResolvedConfig.

use std::collections::HashSet;
use std::path::Path;

use crate::cop::{CopConfig, EnabledState};

use super::globutil::glob_matches;
use super::standard::is_plugin_department;
use super::types::{DepartmentConfig, NewCopsPolicy};
use super::ResolvedConfig;

/// Inputs for RuboCop-style enable_cop? state resolution.
pub(crate) struct EnableInputs {
    pub cop_state: EnabledState,
    pub dept_state: EnabledState,
    pub disabled_by_default: bool,
    pub project_mentioned: bool,
    pub project_enabled_dept: bool,
    /// When resolving under DisabledByDefault + enabled dept, require this
    /// (filters path uses `cop.default_enabled()`; path checks use `true`).
    pub restore_ok: bool,
}

/// Resolve enabled state following RuboCop's enable_cop? precedence.
pub(crate) fn resolve_enabled_state(i: &EnableInputs) -> EnabledState {
    if i.cop_state == EnabledState::True {
        return resolve_true_cop(i);
    }
    if i.cop_state != EnabledState::Unset {
        return i.cop_state;
    }
    if i.dept_state == EnabledState::False {
        return EnabledState::False;
    }
    if i.dept_state == EnabledState::True {
        return resolve_dept_true(i);
    }
    EnabledState::Unset
}

fn resolve_true_cop(i: &EnableInputs) -> EnabledState {
    if !i.disabled_by_default && i.dept_state == EnabledState::False && !i.project_mentioned {
        EnabledState::False
    } else {
        EnabledState::True
    }
}

fn resolve_dept_true(i: &EnableInputs) -> EnabledState {
    if !i.disabled_by_default || (i.project_enabled_dept && i.restore_ok) {
        EnabledState::True
    } else {
        EnabledState::Unset
    }
}

/// Convert resolved state to a bool given NewCops / DisabledByDefault / default.
pub(crate) fn state_to_enabled(
    state: EnabledState,
    new_cops: NewCopsPolicy,
    disabled_by_default: bool,
    default_enabled: bool,
) -> bool {
    match state {
        EnabledState::False => false,
        EnabledState::Pending => new_cops == NewCopsPolicy::Enable && !disabled_by_default,
        EnabledState::Unset => !disabled_by_default && default_enabled,
        EnabledState::True => true,
    }
}

/// Whether any require_known_cops belong to `dept`.
pub(crate) fn dept_has_known_cops(known: &HashSet<String>, dept: &str) -> bool {
    known
        .iter()
        .any(|c| c.starts_with(dept) && c.as_bytes().get(dept.len()) == Some(&b'/'))
}

/// Plugin / core version gates that force a cop off.
///
/// `unset_only`: when true (path checks), unknown-version cops stay if the user
/// set any non-Unset Enabled; when false (filters), only explicit True counts.
pub(crate) fn version_gate_disables(
    cfg: &ResolvedConfig,
    name: &str,
    dept: &str,
    config: Option<&CopConfig>,
    unset_only: bool,
) -> bool {
    if plugin_dept_unloaded(cfg, dept, config) {
        return true;
    }
    if plugin_version_unknown(cfg, name, dept, config, unset_only) {
        return true;
    }
    if core_version_unknown(cfg, name, dept, config, unset_only) {
        return true;
    }
    false
}

fn user_configured(config: Option<&CopConfig>, unset_only: bool) -> bool {
    match config {
        None => false,
        Some(c) if unset_only => c.enabled != EnabledState::Unset,
        Some(c) => c.enabled == EnabledState::True,
    }
}

fn plugin_dept_unloaded(cfg: &ResolvedConfig, dept: &str, config: Option<&CopConfig>) -> bool {
    is_plugin_department(dept)
        && !cfg.require_departments.contains(dept)
        && config.is_none_or(|c| c.enabled != EnabledState::True)
}

fn plugin_version_unknown(
    cfg: &ResolvedConfig,
    name: &str,
    dept: &str,
    config: Option<&CopConfig>,
    unset_only: bool,
) -> bool {
    dept_has_known_cops(&cfg.require_known_cops, dept)
        && cfg.require_departments.contains(dept)
        && !cfg.require_known_cops.contains(name)
        && !user_configured(config, unset_only)
}

fn core_version_unknown(
    cfg: &ResolvedConfig,
    name: &str,
    dept: &str,
    config: Option<&CopConfig>,
    unset_only: bool,
) -> bool {
    !cfg.rubocop_known_cops.is_empty()
        && !is_plugin_department(dept)
        && !cfg.rubocop_known_cops.contains(name)
        && !user_configured(config, unset_only)
}

/// Cross-cop: Style/RedundantConstantBase off when Lint/ConstantResolution is on.
pub(crate) fn redundant_constant_base_blocked(
    name: &str,
    lint_constant_resolution_enabled: bool,
) -> bool {
    name == "Style/RedundantConstantBase" && lint_constant_resolution_enabled
}

pub(crate) fn lint_constant_resolution_on(cfg: &ResolvedConfig) -> bool {
    cfg.cop_configs
        .get("Lint/ConstantResolution")
        .is_some_and(|c| c.enabled == EnabledState::True)
}

/// Effective Include patterns: cop > department > defaults.
pub(crate) fn effective_include<'a>(
    config: Option<&'a CopConfig>,
    dept: Option<&'a DepartmentConfig>,
    defaults: &'a [&str],
) -> Vec<&'a str> {
    match config {
        Some(c) if !c.include.is_empty() => c.include.iter().map(|s| s.as_str()).collect(),
        _ => match dept {
            Some(dc) if !dc.include.is_empty() => dc.include.iter().map(|s| s.as_str()).collect(),
            _ => defaults.to_vec(),
        },
    }
}

/// Effective Exclude patterns: cop > department > defaults.
pub(crate) fn effective_exclude<'a>(
    config: Option<&'a CopConfig>,
    dept: Option<&'a DepartmentConfig>,
    defaults: &'a [&str],
) -> Vec<&'a str> {
    match config {
        Some(c) if !c.exclude.is_empty() => c.exclude.iter().map(|s| s.as_str()).collect(),
        _ => match dept {
            Some(dc) if !dc.exclude.is_empty() => dc.exclude.iter().map(|s| s.as_str()).collect(),
            _ => defaults.to_vec(),
        },
    }
}

pub(crate) fn path_matches_any(patterns: &[&str], path: &Path) -> bool {
    patterns.iter().any(|pat| glob_matches(pat, path))
}

pub(crate) fn global_exclude_hits(cfg: &ResolvedConfig, path: &Path) -> bool {
    cfg.global_excludes
        .iter()
        .any(|pattern| glob_matches(pattern, path))
}

impl ResolvedConfig {
    pub(crate) fn enable_inputs(
        &self,
        name: &str,
        dept: &str,
        config: Option<&CopConfig>,
        restore_ok: bool,
    ) -> EnableInputs {
        let dept_config = self.department_configs.get(dept);
        EnableInputs {
            cop_state: config.map(|c| c.enabled).unwrap_or(EnabledState::Unset),
            dept_state: dept_config
                .map(|dc| dc.enabled)
                .unwrap_or(EnabledState::Unset),
            disabled_by_default: self.disabled_by_default,
            project_mentioned: self.project_mentioned_cops.contains(name),
            project_enabled_dept: self.project_enabled_depts.contains(dept),
            restore_ok,
        }
    }
}
