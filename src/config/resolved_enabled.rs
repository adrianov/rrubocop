//! Per-file cop enablement checks.

use std::path::Path;

use super::resolved_state::{
    effective_exclude, effective_include, global_exclude_hits, lint_constant_resolution_on,
    path_matches_any, redundant_constant_base_blocked, resolve_enabled_state, state_to_enabled,
    version_gate_disables,
};
use super::ResolvedConfig;

impl ResolvedConfig {
    /// Check if a cop is enabled for the given file path.
    ///
    /// Evaluates enabled state, plugin/version gates, global excludes, then
    /// effective Include/Exclude patterns.
    pub fn is_cop_enabled(
        &self,
        name: &str,
        path: &Path,
        default_include: &[&str],
        default_exclude: &[&str],
    ) -> bool {
        let config = self.cop_configs.get(name);
        let dept = name.split('/').next().unwrap_or("");
        let inputs = self.enable_inputs(name, dept, config, true);
        let state = resolve_enabled_state(&inputs);
        if !state_to_enabled(state, self.new_cops, self.disabled_by_default, true) {
            return false;
        }
        if version_gate_disables(self, name, dept, config, true) {
            return false;
        }
        if redundant_constant_base_blocked(name, lint_constant_resolution_on(self)) {
            return false;
        }
        if global_exclude_hits(self, path) {
            return false;
        }
        self.path_passes_patterns(name, path, default_include, default_exclude)
    }

    fn path_passes_patterns(
        &self,
        name: &str,
        path: &Path,
        default_include: &[&str],
        default_exclude: &[&str],
    ) -> bool {
        let config = self.cop_configs.get(name);
        let dept = name.split('/').next().unwrap_or("");
        let dept_config = self.department_configs.get(dept);
        let include = effective_include(config, dept_config, default_include);
        let exclude = effective_exclude(config, dept_config, default_exclude);
        if !include.is_empty() && !path_matches_any(&include, path) {
            return false;
        }
        !path_matches_any(&exclude, path)
    }
}
