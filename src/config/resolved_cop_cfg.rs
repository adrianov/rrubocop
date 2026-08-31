//! CopConfig resolution with AllCops / sibling option injection.

use crate::cop::CopConfig;

use super::resolved_inject::{
    inject_active_support, inject_end_alignment, inject_first_hash_indent, inject_globals,
    inject_hash_alignment, inject_indentation_width, inject_line_length, inject_missing_else,
    inject_quoted_symbols, inject_rack_version, inject_redundant_line_break,
    inject_space_after_comma,
};
use super::ResolvedConfig;

impl ResolvedConfig {
    /// Get the resolved config for a specific cop.
    ///
    /// Injects global AllCops settings and sibling-cop options so individual
    /// cops can access them without special plumbing.
    pub fn cop_config(&self, name: &str) -> CopConfig {
        let mut config = self.cop_configs.get(name).cloned().unwrap_or_default();
        inject_globals(self, &mut config);
        inject_rack_version(self, name, &mut config);
        inject_line_length(self, name, &mut config);
        inject_redundant_line_break(self, name, &mut config);
        inject_active_support(self, name, &mut config);
        inject_hash_alignment(self, name, &mut config);
        inject_first_hash_indent(self, name, &mut config);
        inject_end_alignment(self, name, &mut config);
        inject_indentation_width(self, name, &mut config);
        inject_space_after_comma(self, name, &mut config);
        inject_missing_else(self, name, &mut config);
        inject_quoted_symbols(self, name, &mut config);
        config
    }

    /// Get the resolved config for a cop, applying directory-specific overrides
    /// based on the file path.
    pub fn cop_config_for_file(&self, name: &str, file_path: &std::path::Path) -> CopConfig {
        self.effective_config_for_file(file_path)
            .map_or_else(|| self.cop_config(name), |config| config.cop_config(name))
    }
}
