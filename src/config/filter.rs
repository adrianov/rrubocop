//! Pre-compiled cop Include/Exclude filters.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use globset::GlobSet;
use regex::RegexSet;

use crate::cop::CopRegistry;

use super::globutil::glob_matches;
use super::resolved::ResolvedConfig;

/// Pre-compiled glob filter for a single cop.
pub struct CopFilter {
    pub(crate) enabled: bool,
    pub(crate) include_set: Option<GlobSet>,
    pub(crate) exclude_set: Option<GlobSet>,
    pub(crate) include_re: Option<RegexSet>,
    pub(crate) exclude_re: Option<RegexSet>,
}

impl CopFilter {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_match(&self, path: &Path) -> bool {
        self.enabled && self.is_included(path) && !self.is_excluded(path)
    }

    pub fn is_universal(&self) -> bool {
        self.enabled
            && self.include_set.is_none()
            && self.exclude_set.is_none()
            && self.include_re.is_none()
            && self.exclude_re.is_none()
    }

    pub(crate) fn is_included(&self, path: &Path) -> bool {
        if self.include_set.is_none() && self.include_re.is_none() {
            return true;
        }
        let path_str = path.to_string_lossy();
        if self.include_set.as_ref().is_some_and(|inc| inc.is_match(path)) {
            return true;
        }
        self.include_re
            .as_ref()
            .is_some_and(|re| re.is_match(path_str.as_ref()))
    }

    pub(crate) fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        if self.exclude_set.as_ref().is_some_and(|exc| exc.is_match(path)) {
            return true;
        }
        self.exclude_re
            .as_ref()
            .is_some_and(|re| re.is_match(path_str.as_ref()))
    }
}

/// Pre-compiled filter set for all cops + global excludes.
pub struct CopFilterSet {
    pub(crate) global_exclude: GlobSet,
    pub(crate) global_exclude_patterns: Vec<String>,
    pub(crate) global_exclude_re: Option<RegexSet>,
    pub(crate) filters: Vec<CopFilter>,
    pub(crate) name_to_index: HashMap<String, usize>,
    pub(crate) config_dir: Option<PathBuf>,
    pub(crate) base_dir: Option<PathBuf>,
    pub(crate) scan_root: Option<PathBuf>,
    pub(crate) sub_config_dirs: Vec<PathBuf>,
    pub(crate) universal_cop_indices: Vec<usize>,
    pub(crate) pattern_cop_indices: Vec<usize>,
    pub(crate) migrated_schema_version: Option<String>,
}

impl CopFilterSet {
    pub fn set_scan_root(&mut self, root: PathBuf) {
        self.scan_root = Some(root);
    }

    pub fn build(config: &ResolvedConfig, registry: &CopRegistry) -> Self {
        config.build_cop_filters(registry)
    }

    /// Global excludes only — enough for `--list-target-files` / file discovery.
    pub fn for_discover(config: &ResolvedConfig) -> Self {
        config.build_discover_filters()
    }

    pub fn is_cop_enabled_for_file(&self, name: &str, path: &Path) -> bool {
        self.name_to_index
            .get(name)
            .is_some_and(|&i| self.is_cop_match(i, path))
    }

    pub fn filter_for(&self, name: &str) -> Option<&CopFilter> {
        self.name_to_index.get(name).map(|&i| &self.filters[i])
    }

    pub fn cop_filter(&self, index: usize) -> &CopFilter {
        &self.filters[index]
    }

    pub fn universal_cop_indices(&self) -> &[usize] {
        &self.universal_cop_indices
    }

    pub fn pattern_cop_indices(&self) -> &[usize] {
        &self.pattern_cop_indices
    }

    pub(crate) fn nearest_config_dir(&self, path: &Path) -> Option<&Path> {
        for dir in &self.sub_config_dirs {
            if path.starts_with(dir) {
                return Some(dir.as_path());
            }
        }
        self.config_dir.as_deref()
    }

    pub(crate) fn matches_global_exclude_glob(&self, path: &Path) -> bool {
        self.global_exclude.is_match(path)
            && self
                .global_exclude_patterns
                .iter()
                .any(|pattern| glob_matches(pattern, path))
    }

    fn exclude_re_matches(&self, path: &Path) -> bool {
        self.global_exclude_re
            .as_ref()
            .is_some_and(|re| re.is_match(path.to_string_lossy().as_ref()))
    }

    fn path_globally_excluded_forms(&self, path: &Path) -> bool {
        self.matches_global_exclude_glob(path) || self.exclude_re_matches(path)
    }

    pub fn is_globally_excluded(&self, path: &Path) -> bool {
        if self.path_globally_excluded_forms(path) {
            return true;
        }
        if let Ok(stripped) = path.strip_prefix("./") {
            if self.path_globally_excluded_forms(stripped) {
                return true;
            }
        }
        self.globally_excluded_under_dirs(path)
    }

    fn globally_excluded_under_dirs(&self, path: &Path) -> bool {
        for dir in [&self.base_dir, &self.config_dir].into_iter().flatten() {
            if let Ok(rel) = path.strip_prefix(dir) {
                if self.path_globally_excluded_forms(rel) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_migrated_file(&self, path: &Path) -> bool {
        let Some(ref version) = self.migrated_schema_version else {
            return false;
        };
        let basename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        first_timestamp_leq(basename, version)
    }
}

fn first_timestamp_leq(basename: &str, version: &str) -> bool {
    let bytes = basename.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i - start >= 14 {
            return &basename[start..start + 14] <= version;
        }
    }
    false
}
