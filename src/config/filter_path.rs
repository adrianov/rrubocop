//! Path-form helpers for Include/Exclude matching.

use std::path::{Path, PathBuf};

use globset::GlobSet;
use regex::RegexSet;

use super::filter::{CopFilter, CopFilterSet};

pub(crate) fn rel_to_base_dir<'a>(set: &'a CopFilterSet, path: &'a Path) -> Option<&'a Path> {
    set.base_dir
        .as_deref()
        .filter(|bd| set.config_dir.as_deref() != Some(*bd))
        .and_then(|bd| path.strip_prefix(bd).ok())
}

pub(crate) fn path_forms<'a>(
    set: &'a CopFilterSet,
    path: &'a Path,
) -> (Option<&'a Path>, Option<&'a Path>) {
    let rel_path = set
        .nearest_config_dir(path)
        .and_then(|cd| path.strip_prefix(cd).ok());
    (rel_path, rel_to_base_dir(set, path))
}

fn any_rel_match<'a>(
    forms: impl IntoIterator<Item = Option<&'a Path>>,
    check: impl Fn(&Path) -> bool,
) -> bool {
    forms.into_iter().any(|p| p.is_some_and(&check))
}

pub(crate) fn path_included_any_form(set: &CopFilterSet, filter: &CopFilter, path: &Path) -> bool {
    let (rel_path, rel_to_base) = path_forms(set, path);
    let rel_to_scan = set
        .scan_root
        .as_deref()
        .and_then(|root| path.strip_prefix(root).ok());
    let stripped = path.strip_prefix("./").ok();
    filter.is_included(path)
        || any_rel_match([rel_path, rel_to_base, rel_to_scan, stripped], |p| {
            filter.is_included(p)
        })
}

pub(crate) fn path_excluded_cop_forms(set: &CopFilterSet, filter: &CopFilter, path: &Path) -> bool {
    let (rel_path, rel_to_base) = path_forms(set, path);
    let stripped = path.strip_prefix("./").ok();
    filter.is_excluded(path)
        || any_rel_match([rel_path, rel_to_base, stripped], |p| filter.is_excluded(p))
}

fn exclude_rel_forms<'a>(
    set: &'a CopFilterSet,
    path: &'a Path,
) -> [Option<&'a Path>; 4] {
    let nearest_dir = set.nearest_config_dir(path);
    let rel_to_nearest = nearest_dir.and_then(|cd| path.strip_prefix(cd).ok());
    let rel_to_root = set
        .config_dir
        .as_deref()
        .filter(|root| nearest_dir.is_some_and(|n| n != *root))
        .and_then(|cd| path.strip_prefix(cd).ok());
    let rel_to_base = rel_to_base_dir(set, path);
    let stripped = path.strip_prefix("./").ok();
    [rel_to_nearest, rel_to_root, rel_to_base, stripped]
}

pub(crate) fn filter_excluded_any_form(
    set: &CopFilterSet,
    filter: &CopFilter,
    path: &Path,
) -> bool {
    filter.is_excluded(path)
        || any_rel_match(exclude_rel_forms(set, path), |p| filter.is_excluded(p))
}

fn glob_matches_forms(
    set: &GlobSet,
    path: &Path,
    rel_path: Option<&Path>,
    rel_to_base: Option<&Path>,
) -> bool {
    set.is_match(path) || any_rel_match([rel_path, rel_to_base], |rel| set.is_match(rel))
}

fn re_matches_forms(
    re: &RegexSet,
    path: &Path,
    rel_path: Option<&Path>,
    rel_to_base: Option<&Path>,
) -> bool {
    let path_str = path.to_string_lossy();
    re.is_match(path_str.as_ref())
        || any_rel_match([rel_path, rel_to_base], |rel| {
            re.is_match(&rel.to_string_lossy())
        })
}

pub(crate) fn include_matches_forms(
    include_set: &Option<GlobSet>,
    include_re: &Option<RegexSet>,
    path: &Path,
    rel_path: Option<&Path>,
    rel_to_base: Option<&Path>,
) -> bool {
    let glob_ok = include_set
        .as_ref()
        .is_some_and(|inc| glob_matches_forms(inc, path, rel_path, rel_to_base));
    let re_ok = include_re
        .as_ref()
        .is_some_and(|re| re_matches_forms(re, path, rel_path, rel_to_base));
    glob_ok || re_ok
}

pub(crate) fn exclude_set_matches(
    exclude_set: &Option<GlobSet>,
    path: &Path,
    rel_path: Option<&Path>,
    rel_to_base: Option<&Path>,
) -> bool {
    exclude_set
        .as_ref()
        .is_some_and(|exc| glob_matches_forms(exc, path, rel_path, rel_to_base))
}

pub(crate) fn exclude_re_matches_forms(
    exclude_re: &Option<RegexSet>,
    path: &Path,
    rel_path: Option<&Path>,
    rel_to_base: Option<&Path>,
) -> bool {
    exclude_re
        .as_ref()
        .is_some_and(|re| re_matches_forms(re, path, rel_path, rel_to_base))
}

pub(crate) fn path_under_dir(file: &Path, dir: &Path) -> bool {
    for f in path_prefix_variants(file) {
        for d in path_prefix_variants(dir) {
            if f.starts_with(&d) {
                return true;
            }
        }
    }
    false
}

fn path_prefix_variants(path: &Path) -> [PathBuf; 2] {
    let primary = path.to_path_buf();
    let Some(text) = path.to_str() else {
        return [primary.clone(), primary];
    };
    let alt = if let Some(rest) = text.strip_prefix("/private") {
        PathBuf::from(rest)
    } else if text.starts_with("/var/") || text.starts_with("/tmp/") {
        PathBuf::from(format!("/private{text}"))
    } else {
        primary.clone()
    };
    [primary, alt]
}
