//! Path matching for CopFilterSet (Include/Exclude forms).

use std::path::Path;

use crate::cop::CopConfig;

use super::filter::{CopFilterSet};
use super::filter_path::{
    exclude_re_matches_forms, exclude_set_matches, filter_excluded_any_form, include_matches_forms,
    path_excluded_cop_forms, path_forms, path_included_any_form,
};
use super::globutil::{build_glob_set, build_regex_set};

fn is_cop_match_impl(set: &CopFilterSet, index: usize, path: &Path) -> bool {
    let filter = &set.filters[index];
    if !filter.enabled {
        return false;
    }
    path_included_any_form(set, filter, path) && !path_excluded_cop_forms(set, filter, path)
}

fn is_cop_excluded_impl(set: &CopFilterSet, index: usize, path: &Path) -> bool {
    filter_excluded_any_form(set, &set.filters[index], path)
}

fn build_path_sets(
    include_pats: &[&str],
    exclude_pats: &[&str],
) -> (
    Option<globset::GlobSet>,
    Option<globset::GlobSet>,
    Option<regex::RegexSet>,
    Option<regex::RegexSet>,
) {
    (
        build_glob_set(include_pats),
        build_glob_set(exclude_pats),
        build_regex_set(include_pats),
        build_regex_set(exclude_pats),
    )
}

fn path_passes_include(
    include_set: &Option<globset::GlobSet>,
    include_re: &Option<regex::RegexSet>,
    path: &Path,
    rel_path: Option<&Path>,
    rel_to_base: Option<&Path>,
) -> bool {
    if include_set.is_none() && include_re.is_none() {
        return true;
    }
    include_matches_forms(include_set, include_re, path, rel_path, rel_to_base)
}

fn path_passes_exclude(
    exclude_set: &Option<globset::GlobSet>,
    exclude_re: &Option<regex::RegexSet>,
    path: &Path,
    rel_path: Option<&Path>,
    rel_to_base: Option<&Path>,
) -> bool {
    !exclude_set_matches(exclude_set, path, rel_path, rel_to_base)
        && !exclude_re_matches_forms(exclude_re, path, rel_path, rel_to_base)
}

fn is_path_matched_impl(set: &CopFilterSet, cop_config: &CopConfig, path: &Path) -> bool {
    let include_pats: Vec<&str> = cop_config.include.iter().map(|s| s.as_str()).collect();
    let exclude_pats: Vec<&str> = cop_config.exclude.iter().map(|s| s.as_str()).collect();
    let (include_set, exclude_set, include_re, exclude_re) =
        build_path_sets(&include_pats, &exclude_pats);
    let (rel_path, rel_to_base) = path_forms(set, path);
    path_passes_include(&include_set, &include_re, path, rel_path, rel_to_base)
        && path_passes_exclude(&exclude_set, &exclude_re, path, rel_path, rel_to_base)
}

impl CopFilterSet {
    pub fn is_cop_match(&self, index: usize, path: &Path) -> bool {
        is_cop_match_impl(self, index, path)
    }

    pub fn is_cop_excluded(&self, index: usize, path: &Path) -> bool {
        is_cop_excluded_impl(self, index, path)
    }

    pub fn is_path_matched_by_cop_config(&self, cop_config: &CopConfig, path: &Path) -> bool {
        is_path_matched_impl(self, cop_config, path)
    }
}
