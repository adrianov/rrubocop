//! Parallel file linting with discovery/lint pipeline and fail-fast.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Mutex;
use std::thread;

use anyhow::Result;
use rayon::prelude::*;

use crate::cache::{Cache, CacheSettings};
use crate::cli::AutocorrectMode;
use crate::config::{CopFilterSet, ResolvedConfig};
use crate::cop::registry::CopRegistry;
use crate::diagnostic::Diagnostic;
use crate::fs::discover_emitting;
use crate::parse::source::SourceFile;

use super::engine::lint_source;
use super::RunPrep;

pub(super) struct LintBatch {
    pub diagnostics: Vec<Diagnostic>,
    pub inspected: Vec<PathBuf>,
    pub discovered_count: usize,
}

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

/// Lint while discovery is still walking: paths are queued as found.
pub(super) fn lint_pipeline(
    prep: &RunPrep,
    paths: &[PathBuf],
    config: &ResolvedConfig,
    registry: &CopRegistry,
    on_discovered: impl FnOnce(usize),
    on_file: impl Fn(&[Diagnostic]) + Sync,
) -> Result<LintBatch> {
    let diagnostics = Mutex::new(Vec::new());
    let inspected = Mutex::new(Vec::new());
    let stop = AtomicBool::new(false);
    let fail_count = AtomicU32::new(0);
    let settings = cache_settings(prep);
    let discovered_count = run_discover_lint(
        prep,
        paths,
        config,
        registry,
        settings,
        &diagnostics,
        &inspected,
        &stop,
        &fail_count,
        on_discovered,
        &on_file,
    )?;
    Ok(LintBatch {
        diagnostics: take_sorted(diagnostics),
        inspected: inspected.into_inner().unwrap(),
        discovered_count,
    })
}

fn run_discover_lint(
    prep: &RunPrep,
    paths: &[PathBuf],
    config: &ResolvedConfig,
    registry: &CopRegistry,
    settings: CacheSettings<'_>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    inspected: &Mutex<Vec<PathBuf>>,
    stop: &AtomicBool,
    fail_count: &AtomicU32,
    on_discovered: impl FnOnce(usize),
    on_file: &(impl Fn(&[Diagnostic]) + Sync),
) -> Result<usize> {
    let (tx, rx) = mpsc::channel::<PathBuf>();
    let rx = Mutex::new(rx);
    thread::scope(|scope| {
        let discover = scope.spawn(|| discover_paths(paths, prep, stop, tx));
        let lint = scope.spawn(|| {
            run_workers(
                &rx,
                prep,
                config,
                registry,
                settings,
                diagnostics,
                inspected,
                stop,
                fail_count,
                on_file,
            )
        });
        let count = discover.join().unwrap()?;
        on_discovered(count);
        lint.join().unwrap()?;
        Ok(count)
    })
}

fn discover_paths(
    paths: &[PathBuf],
    prep: &RunPrep,
    stop: &AtomicBool,
    tx: mpsc::Sender<PathBuf>,
) -> Result<usize> {
    let result = discover_emitting(paths, &prep.filters, prep.force_exclusion, stop, |p| {
        let _ = tx.send(p);
    });
    drop(tx);
    result
}

fn run_workers(
    rx: &Mutex<Receiver<PathBuf>>,
    prep: &RunPrep,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    settings: CacheSettings<'_>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    inspected: &Mutex<Vec<PathBuf>>,
    stop: &AtomicBool,
    fail_count: &AtomicU32,
    on_file: &(impl Fn(&[Diagnostic]) + Sync),
) -> Result<()> {
    let workers = rayon::current_num_threads().max(1);
    (0..workers).into_par_iter().try_for_each(|_| {
        worker_loop(
            rx,
            prep,
            config,
            registry,
            settings,
            diagnostics,
            inspected,
            stop,
            fail_count,
            on_file,
        )
    })
}

fn worker_loop(
    rx: &Mutex<Receiver<PathBuf>>,
    prep: &RunPrep,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    settings: CacheSettings<'_>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    inspected: &Mutex<Vec<PathBuf>>,
    stop: &AtomicBool,
    fail_count: &AtomicU32,
    on_file: &impl Fn(&[Diagnostic]),
) -> Result<()> {
    loop {
        if stop.load(Ordering::Relaxed) {
            while rx.lock().unwrap().try_recv().is_ok() {}
            return Ok(());
        }
        let path = { rx.lock().unwrap().recv() };
        let Ok(path) = path else {
            return Ok(());
        };
        lint_path_job(
            &path,
            prep,
            config,
            registry,
            settings,
            diagnostics,
            inspected,
            stop,
            fail_count,
            on_file,
        )?;
    }
}

fn bump_fail_fast(prep: &RunPrep, add: u32, fail_count: &AtomicU32, stop: &AtomicBool) {
    if prep.fail_fast_limit == 0 || add == 0 {
        return;
    }
    let total = fail_count.fetch_add(add, Ordering::Relaxed) + add;
    if total >= prep.fail_fast_limit {
        stop.store(true, Ordering::Relaxed);
    }
}

fn apply_diags(
    path: &Path,
    diags: &mut Vec<Diagnostic>,
    inspected: &Mutex<Vec<PathBuf>>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    on_file: &impl Fn(&[Diagnostic]),
) {
    on_file(diags);
    inspected.lock().unwrap().push(path.to_path_buf());
    diagnostics.lock().unwrap().append(diags);
}

fn lint_path_job(
    path: &Path,
    prep: &RunPrep,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    settings: CacheSettings<'_>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    inspected: &Mutex<Vec<PathBuf>>,
    stop: &AtomicBool,
    fail_count: &AtomicU32,
    on_file: &impl Fn(&[Diagnostic]),
) -> Result<()> {
    if stop.load(Ordering::Relaxed) {
        return Ok(());
    }
    let mut diags = lint_file(
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
    bump_fail_fast(
        prep,
        counted_offenses(&diags, prep.fail_fast_uncorrected_only),
        fail_count,
        stop,
    );
    apply_diags(path, &mut diags, inspected, diagnostics, on_file);
    Ok(())
}

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
