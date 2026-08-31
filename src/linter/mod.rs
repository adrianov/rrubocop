//! Parallel lint runner.

mod engine;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use rayon::prelude::*;

use crate::cache::{Cache, CacheSettings};
use crate::cli::{Args, AutocorrectMode};
use crate::config::{CopFilterSet, ResolvedConfig};
use crate::cop::registry::CopRegistry;
use crate::diagnostic::{Diagnostic, Severity};
use crate::fs::DiscoveredFiles;
use crate::parse::source::SourceFile;

pub use engine::lint_source;

pub struct LintResult {
    pub diagnostics: Vec<Diagnostic>,
    pub files: Vec<PathBuf>,
}

pub fn run_linter(
    args: &Args,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    discovered: &DiscoveredFiles,
) -> Result<LintResult> {
    let prep = prepare_run(args, config, registry, discovered);
    let diagnostics = lint_all_files(&prep, config, registry)?;
    Ok(LintResult {
        diagnostics,
        files: prep.files,
    })
}

struct RunPrep {
    filters: CopFilterSet,
    only: Option<Vec<String>>,
    except: Vec<String>,
    mode: AutocorrectMode,
    files: Vec<PathBuf>,
    cache: Option<Cache>,
    config_fp: Vec<u8>,
    only_key: String,
    except_key: String,
    ignore_disable: bool,
    force_default_config: bool,
    fail_fast: bool,
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
        fail_fast: args.fail_fast,
    }
}

fn lint_all_files(
    prep: &RunPrep,
    config: &ResolvedConfig,
    registry: &CopRegistry,
) -> Result<Vec<Diagnostic>> {
    let diagnostics = Mutex::new(Vec::new());
    let stop = AtomicBool::new(false);
    let settings = CacheSettings {
        only: &prep.only_key,
        except: &prep.except_key,
        ignore_disable: prep.ignore_disable,
        force_default_config: prep.force_default_config,
        config_fingerprint: &prep.config_fp,
    };
    prep.files.par_iter().try_for_each(|path| -> Result<()> {
        lint_path_job(path, prep, config, registry, settings, &diagnostics, &stop)
    })?;
    let mut diags = diagnostics.into_inner().unwrap();
    diags.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(diags)
}

fn lint_path_job(
    path: &Path,
    prep: &RunPrep,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    settings: CacheSettings<'_>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    stop: &AtomicBool,
) -> Result<()> {
    if stop.load(Ordering::Relaxed) {
        return Ok(());
    }
    let mut file_diags = lint_file(
        path,
        config,
        registry,
        &prep.filters,
        prep.only.as_deref(),
        &prep.except,
        prep.mode,
        prep.ignore_disable,
        prep.cache.as_ref(),
        settings,
    )?;
    if !file_diags.is_empty() && prep.fail_fast {
        stop.store(true, Ordering::Relaxed);
    }
    diagnostics.lock().unwrap().append(&mut file_diags);
    Ok(())
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

#[allow(clippy::too_many_arguments)]
fn lint_file(
    path: &Path,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    filters: &CopFilterSet,
    only: Option<&[String]>,
    except: &[String],
    mode: AutocorrectMode,
    ignore_disable: bool,
    cache: Option<&Cache>,
    settings: CacheSettings<'_>,
) -> Result<Vec<Diagnostic>> {
    let source = SourceFile::from_path(path)?;
    if let Some(hit) = cache_get(cache, path, &source, settings) {
        return Ok(hit);
    }
    let diags = lint_source(
        &source,
        config,
        registry,
        filters,
        only,
        except,
        mode,
        ignore_disable,
        true,
    )?;
    cache_put(cache, path, &source, settings, &diags);
    Ok(diags)
}

fn cache_get(
    cache: Option<&Cache>,
    path: &Path,
    source: &SourceFile,
    settings: CacheSettings<'_>,
) -> Option<Vec<Diagnostic>> {
    let cache = cache?;
    cache.get(&cache.file_key(path, source.as_bytes(), settings))
}

fn cache_put(
    cache: Option<&Cache>,
    path: &Path,
    source: &SourceFile,
    settings: CacheSettings<'_>,
    diags: &[Diagnostic],
) {
    let Some(cache) = cache else {
        return;
    };
    cache.store(&cache.file_key(path, source.as_bytes(), settings), diags);
}

pub fn should_fail(diagnostics: &[Diagnostic], fail_level: &str) -> bool {
    let Some(min) = Severity::from_str(fail_level) else {
        return !diagnostics.is_empty();
    };
    diagnostics.iter().any(|d| d.severity >= min)
}
