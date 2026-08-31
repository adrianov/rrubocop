//! Parallel file linting and fail-fast batching.

use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use anyhow::Result;
use rayon::prelude::*;

use crate::cache::{Cache, CacheSettings};
use crate::cli::AutocorrectMode;
use crate::config::{CopFilterSet, ResolvedConfig};
use crate::cop::registry::CopRegistry;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

use super::engine::lint_source;
use super::RunPrep;

fn cache_settings(prep: &RunPrep) -> CacheSettings<'_> {
    CacheSettings {
        only: &prep.only_key,
        except: &prep.except_key,
        ignore_disable: prep.ignore_disable,
        force_default_config: prep.force_default_config,
        config_fingerprint: &prep.config_fp,
    }
}

fn take_sorted(diagnostics: Mutex<Vec<Diagnostic>>) -> Vec<Diagnostic> {
    let mut diags = diagnostics.into_inner().unwrap();
    diags.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    diags
}

fn counted_offenses(diags: &[Diagnostic], uncorrected_only: bool) -> u32 {
    let n = if uncorrected_only {
        diags.iter().filter(|d| !d.corrected).count()
    } else {
        diags.len()
    };
    n as u32
}

pub(super) fn lint_all_files(
    prep: &RunPrep,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    on_file: impl Fn(&[Diagnostic]) + Sync,
) -> Result<Vec<Diagnostic>> {
    let diagnostics = Mutex::new(Vec::new());
    let stop = AtomicBool::new(false);
    let fail_count = AtomicU32::new(0);
    let settings = cache_settings(prep);
    if prep.fail_fast_limit == 0 {
        lint_unlimited(
            prep,
            config,
            registry,
            settings,
            &diagnostics,
            &stop,
            &fail_count,
            &on_file,
        )?;
    } else {
        lint_fail_fast_batches(
            prep,
            config,
            registry,
            settings,
            &diagnostics,
            &stop,
            &fail_count,
            &on_file,
        )?;
    }
    Ok(take_sorted(diagnostics))
}

fn lint_unlimited(
    prep: &RunPrep,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    settings: CacheSettings<'_>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    stop: &AtomicBool,
    fail_count: &AtomicU32,
    on_file: &(impl Fn(&[Diagnostic]) + Sync),
) -> Result<()> {
    prep.files.par_iter().try_for_each(|path| {
        lint_path_job(
            path,
            prep,
            config,
            registry,
            settings,
            diagnostics,
            stop,
            fail_count,
            on_file,
        )
    })
}

fn lint_fail_fast_batches(
    prep: &RunPrep,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    settings: CacheSettings<'_>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    stop: &AtomicBool,
    fail_count: &AtomicU32,
    on_file: &(impl Fn(&[Diagnostic]) + Sync),
) -> Result<()> {
    let threads = rayon::current_num_threads().max(1);
    let mut start = 0;
    while start < prep.files.len() {
        let used = fail_count.load(Ordering::Relaxed);
        if used >= prep.fail_fast_limit {
            break;
        }
        let slots = (prep.fail_fast_limit - used) as usize;
        let end = (start + slots.min(threads)).min(prep.files.len());
        prep.files[start..end].par_iter().try_for_each(|path| {
            lint_path_job(
                path,
                prep,
                config,
                registry,
                settings,
                diagnostics,
                stop,
                fail_count,
                on_file,
            )
        })?;
        start = end;
    }
    Ok(())
}

fn lint_path_job(
    path: &Path,
    prep: &RunPrep,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    settings: CacheSettings<'_>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    stop: &AtomicBool,
    fail_count: &AtomicU32,
    on_file: &impl Fn(&[Diagnostic]),
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
    let add = counted_offenses(&file_diags, prep.fail_fast_uncorrected_only);
    if prep.fail_fast_limit > 0 && add > 0 {
        let total = fail_count.fetch_add(add, Ordering::Relaxed) + add;
        if total >= prep.fail_fast_limit {
            stop.store(true, Ordering::Relaxed);
        }
    }
    on_file(&file_diags);
    diagnostics.lock().unwrap().append(&mut file_diags);
    Ok(())
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
