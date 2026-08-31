//! Parallel lint runner.

mod engine;
mod parallel;

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cache::Cache;
use crate::cli::{Args, AutocorrectMode};
use crate::config::{CopFilterSet, ResolvedConfig};
use crate::cop::registry::CopRegistry;
use crate::diagnostic::{Diagnostic, Severity};
use crate::fs::DiscoveredFiles;

pub use engine::lint_source;

pub struct LintResult {
    pub diagnostics: Vec<Diagnostic>,
    pub files: Vec<PathBuf>,
}

pub(crate) struct RunPrep {
    pub(crate) filters: CopFilterSet,
    pub(crate) only: Option<Vec<String>>,
    pub(crate) except: Vec<String>,
    pub(crate) mode: AutocorrectMode,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) cache: Option<Cache>,
    pub(crate) config_fp: Vec<u8>,
    pub(crate) only_key: String,
    pub(crate) except_key: String,
    pub(crate) ignore_disable: bool,
    pub(crate) force_default_config: bool,
    /// `0` = unlimited; otherwise stop once counted offenses reach this.
    pub(crate) fail_fast_limit: u32,
    /// When autocorrecting, only non-corrected offenses count toward the limit.
    pub(crate) fail_fast_uncorrected_only: bool,
}

pub fn run_linter(
    args: &Args,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    discovered: &DiscoveredFiles,
) -> Result<LintResult> {
    run_linter_with(args, config, registry, discovered, |_| {}, |_| {})
}

/// Like [`run_linter`], with `on_start(file_count)` before work and `on_file`
/// once per finished path (completion order; not sorted).
pub fn run_linter_with(
    args: &Args,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    discovered: &DiscoveredFiles,
    on_start: impl FnOnce(usize),
    on_file: impl Fn(&[Diagnostic]) + Sync,
) -> Result<LintResult> {
    let prep = prepare_run(args, config, registry, discovered);
    on_start(prep.files.len());
    let batch = parallel::lint_all_files(&prep, config, registry, on_file)?;
    Ok(LintResult {
        diagnostics: batch.diagnostics,
        // Only paths actually linted (fail-fast may stop early).
        files: batch.inspected,
    })
}

fn prepare_run(
    args: &Args,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    discovered: &DiscoveredFiles,
) -> RunPrep {
    let filters = CopFilterSet::build(config, registry);
    let only = optional_cop_list(&args.only, registry);
    let except = expand_cop_list(&args.except, registry);
    let mode = args.autocorrect_mode();
    let only_key = only.as_ref().map(|v| v.join(",")).unwrap_or_default();
    RunPrep {
        files: select_files(discovered, &filters, args.force_exclusion),
        cache: open_result_cache(mode, args.no_cache),
        config_fp: config.cache_fingerprint(),
        except_key: except.join(","),
        only_key,
        only,
        except,
        mode,
        filters,
        ignore_disable: args.ignore_disable_comments,
        force_default_config: args.force_default_config,
        fail_fast_limit: args.fail_fast,
        fail_fast_uncorrected_only: mode != AutocorrectMode::Off,
    }
}

fn optional_cop_list(list: &[String], registry: &CopRegistry) -> Option<Vec<String>> {
    (!list.is_empty()).then(|| expand_cop_list(list, registry))
}

fn select_files(
    discovered: &DiscoveredFiles,
    filters: &CopFilterSet,
    force_exclusion: bool,
) -> Vec<PathBuf> {
    discovered
        .files
        .iter()
        .filter(|f| include_file(f, discovered, filters, force_exclusion))
        .cloned()
        .collect()
}

fn include_file(
    path: &Path,
    discovered: &DiscoveredFiles,
    filters: &CopFilterSet,
    force_exclusion: bool,
) -> bool {
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if discovered.explicit.contains(&canon) && !force_exclusion {
        return true;
    }
    !filters.is_globally_excluded(path)
}

fn open_result_cache(mode: AutocorrectMode, no_cache: bool) -> Option<Cache> {
    if mode != AutocorrectMode::Off {
        return None;
    }
    let cache = Cache::open(no_cache)?;
    cache.prune();
    Some(cache)
}

fn expand_cop_list(list: &[String], registry: &CopRegistry) -> Vec<String> {
    let mut out = Vec::new();
    for item in list {
        if item.contains('/') {
            out.push(item.clone());
            continue;
        }
        out.extend(
            registry
                .names()
                .into_iter()
                .filter(|name| name.starts_with(&format!("{item}/")))
                .map(str::to_string),
        );
    }
    out
}

pub fn should_fail(diagnostics: &[Diagnostic], fail_level: &str) -> bool {
    let Some(min) = Severity::from_str(fail_level) else {
        return !diagnostics.is_empty();
    };
    diagnostics.iter().any(|d| d.severity >= min)
}
