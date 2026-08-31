//! Parallel lint runner.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Result;
use rayon::prelude::*;

use crate::cli::{Args, AutocorrectMode};
use crate::config::{CopFilterSet, ResolvedConfig};
use crate::cop::registry::CopRegistry;
use crate::cop::walker::BatchedWalker;
use crate::cop::{Cop, CopConfig};
use crate::correction::CorrectionSet;
use crate::diagnostic::{Diagnostic, Severity};
use crate::fs::DiscoveredFiles;
use crate::parse;
use crate::parse::codemap::CodeMap;
use crate::parse::directives;
use crate::parse::source::SourceFile;

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
    let filters = CopFilterSet::build(config, registry);
    let only: Option<Vec<String>> = if args.only.is_empty() {
        None
    } else {
        Some(expand_cop_list(&args.only, registry))
    };
    let except: Vec<String> = expand_cop_list(&args.except, registry);
    let mode = args.autocorrect_mode();
    let fail_fast = args.fail_fast;
    let ignore_disable = args.ignore_disable_comments;
    let force_exclusion = args.force_exclusion;

    let files: Vec<PathBuf> = discovered
        .files
        .iter()
        .filter(|f| {
            let canon = f.canonicalize().unwrap_or_else(|_| (*f).clone());
            let is_explicit = discovered.explicit.contains(&canon);
            if is_explicit && !force_exclusion {
                return true;
            }
            !filters.is_globally_excluded(f)
        })
        .cloned()
        .collect();

    let diagnostics = Mutex::new(Vec::new());
    let stop = std::sync::atomic::AtomicBool::new(false);

    files.par_iter().try_for_each(|path| -> Result<()> {
        if stop.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
        let mut file_diags = lint_file(
            path,
            config,
            registry,
            &filters,
            only.as_deref(),
            &except,
            mode,
            ignore_disable,
        )?;
        if !file_diags.is_empty() && fail_fast {
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        diagnostics.lock().unwrap().append(&mut file_diags);
        Ok(())
    })?;

    let mut diags = diagnostics.into_inner().unwrap();
    diags.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(LintResult {
        diagnostics: diags,
        files,
    })
}

fn expand_cop_list(list: &[String], registry: &CopRegistry) -> Vec<String> {
    let mut out = Vec::new();
    for item in list {
        if item.contains('/') {
            out.push(item.clone());
        } else {
            // Department shorthand
            for name in registry.names() {
                if name.starts_with(&format!("{item}/")) {
                    out.push(name.to_string());
                }
            }
        }
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
) -> Result<Vec<Diagnostic>> {
    let source = SourceFile::from_path(path)?;
    lint_source(
        &source,
        config,
        registry,
        filters,
        only,
        except,
        mode,
        ignore_disable,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn lint_source(
    source: &SourceFile,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    filters: &CopFilterSet,
    only: Option<&[String]>,
    except: &[String],
    mode: AutocorrectMode,
    ignore_disable: bool,
    write_autocorrect: bool,
) -> Result<Vec<Diagnostic>> {
    let path = source.path.as_path();
    let mut active: Vec<(&dyn Cop, CopConfig, usize)> = Vec::new();
    for (idx, cop) in registry.cops().iter().enumerate() {
        let name = cop.name();
        if let Some(only) = only
            && !only.iter().any(|o| o == name)
        {
            continue;
        }
        if except.iter().any(|e| e == name) {
            continue;
        }
        if !filters.is_cop_enabled_for_file(name, path) {
            continue;
        }
        active.push((&**cop, config.cop_config(name), idx));
    }

    if active.is_empty() {
        return Ok(Vec::new());
    }

    let tree = parse::parse_ruby(source)?;
    let code_map = CodeMap::from_tree(tree.root_node(), source.as_bytes());
    let dir_map = if ignore_disable {
        None
    } else {
        let text = String::from_utf8_lossy(source.as_bytes());
        Some(directives::parse(&text))
    };

    let mut diagnostics = Vec::new();
    let mut corrections = if mode != AutocorrectMode::Off {
        Some(Vec::new())
    } else {
        None
    };

    // Phase 1: check_lines
    for (cop, cfg, idx) in &active {
        let allow_corr = match mode {
            AutocorrectMode::Off => false,
            AutocorrectMode::Safe => cop.supports_autocorrect() && cop.safe_autocorrect(),
            AutocorrectMode::All => cop.supports_autocorrect(),
        };
        let mut corr_buf = if allow_corr {
            Some(Vec::new())
        } else {
            None
        };
        let before = diagnostics.len();
        cop.check_lines(
            source,
            cfg,
            &mut diagnostics,
            corr_buf.as_mut(),
        );
        stamp_cop_index(&mut corr_buf, *idx);
        if let (Some(all), Some(buf)) = (&mut corrections, corr_buf) {
            all.extend(buf);
        }
        apply_severity_override(&mut diagnostics[before..], cfg);
    }

    // Phase 2: check_source
    for (cop, cfg, idx) in &active {
        let allow_corr = match mode {
            AutocorrectMode::Off => false,
            AutocorrectMode::Safe => cop.supports_autocorrect() && cop.safe_autocorrect(),
            AutocorrectMode::All => cop.supports_autocorrect(),
        };
        let mut corr_buf = if allow_corr {
            Some(Vec::new())
        } else {
            None
        };
        let before = diagnostics.len();
        cop.check_source(
            source,
            &tree,
            &code_map,
            cfg,
            &mut diagnostics,
            corr_buf.as_mut(),
        );
        stamp_cop_index(&mut corr_buf, *idx);
        if let (Some(all), Some(buf)) = (&mut corrections, corr_buf) {
            all.extend(buf);
        }
        apply_severity_override(&mut diagnostics[before..], cfg);
    }

    // Phase 3: AST walk
    let node_cops: Vec<&dyn Cop> = active.iter().map(|(c, _, _)| *c).collect();
    let node_cfgs: Vec<&CopConfig> = active.iter().map(|(_, c, _)| c).collect();
    // For node phase we need per-cop corrections with indices — run individually for cops with interested kinds or all
    let walker = BatchedWalker::new(node_cops, node_cfgs);
    let mut node_corr = if mode != AutocorrectMode::Off {
        Some(Vec::new())
    } else {
        None
    };
    let before = diagnostics.len();
    walker.walk(
        source,
        tree.root_node(),
        &mut diagnostics,
        node_corr.as_mut(),
    );
    // Stamp corrections with first matching cop index (approximate)
    if let Some(ref mut buf) = node_corr {
        for c in buf.iter_mut() {
            if let Some(idx) = registry.index_of(c.cop_name) {
                c.cop_index = idx;
            }
        }
        if let Some(ref mut all) = corrections {
            all.append(buf);
        }
    }
    let _ = before;

    // Filter disable directives
    if let Some(ref dirs) = dir_map {
        diagnostics.retain(|d| !dirs.suppresses(&d.cop_name, d.location.line));
    }

    // Apply autocorrect
    if write_autocorrect
        && let Some(corrs) = corrections
        && !corrs.is_empty()
    {
        let set = CorrectionSet::from_vec(corrs);
        if !set.is_empty() {
            let new_bytes = set.apply(source.as_bytes());
            std::fs::write(&source.path, &new_bytes)?;
        }
    }

    Ok(diagnostics)
}

fn stamp_cop_index(corr: &mut Option<Vec<crate::correction::Correction>>, idx: usize) {
    if let Some(buf) = corr {
        for c in buf {
            c.cop_index = idx;
        }
    }
}

fn apply_severity_override(diags: &mut [Diagnostic], cfg: &CopConfig) {
    if let Some(sev) = cfg.severity {
        for d in diags {
            d.severity = sev;
        }
    }
}

pub fn should_fail(diagnostics: &[Diagnostic], fail_level: &str) -> bool {
    let Some(min) = Severity::from_str(fail_level) else {
        return !diagnostics.is_empty();
    };
    diagnostics.iter().any(|d| d.severity >= min)
}
