//! Per-file cop execution (line / source / AST phases).

mod syntax_gate;

use anyhow::Result;

use crate::cli::AutocorrectMode;
use crate::config::{CopFilterSet, ResolvedConfig};
use crate::cop::registry::CopRegistry;
use crate::cop::walker::BatchedWalker;
use crate::cop::{Cop, CopConfig};
use crate::correction::{Correction, CorrectionSet};
use crate::diagnostic::Diagnostic;
use crate::parse;
use crate::parse::codemap::CodeMap;
use crate::parse::directives;
use crate::parse::source::SourceFile;

type ActiveCop<'a> = syntax_gate::ActiveCop<'a>;

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
    let active = select_active(source, config, registry, filters, only, except);
    if active.is_empty() {
        return Ok(Vec::new());
    }
    let tree = parse::parse_ruby(source)?;
    let mut diagnostics = Vec::new();
    let mut corrections = corr_bucket(mode);
    if !syntax_gate::run(
        source,
        &tree,
        config,
        &active,
        mode,
        &mut diagnostics,
        &mut corrections,
    ) {
        run_non_syntax(
            source,
            &tree,
            &active,
            registry,
            mode,
            &mut diagnostics,
            &mut corrections,
        );
        if write_autocorrect {
            write_fixes(source, corrections)?;
        }
    }
    filter_directives(source, ignore_disable, &mut diagnostics);
    finalize_offenses(source, config, registry, &mut diagnostics);
    Ok(diagnostics)
}

/// RuboCop MessageAnnotator + clang source line / correctable flags.
fn finalize_offenses(
    source: &SourceFile,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    diagnostics: &mut [Diagnostic],
) {
    for d in diagnostics.iter_mut() {
        enrich_offense(source, config, registry, d);
    }
}

fn enrich_offense(
    source: &SourceFile,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    d: &mut Diagnostic,
) {
    if let Some(cop) = registry.get(d.cop_name.as_str()) {
        if !cop.supports_autocorrect() {
            d.correctable = false;
        }
    }
    fill_source_highlight(source, d);
    let cop_cfg = config.cop_config(&d.cop_name);
    let details = cop_cfg.options.get("Details").and_then(|v| v.as_str());
    let style_guide = style_guide_url(config, &cop_cfg);
    let raw = strip_cop_prefix(&d.message, &d.cop_name);
    d.message = crate::diagnostic::annotate_offense_message(
        raw,
        &d.cop_name,
        config.display_cop_names,
        config.extra_details,
        details,
        config.display_style_guide,
        style_guide.as_deref(),
    );
}

fn fill_source_highlight(source: &SourceFile, d: &mut Diagnostic) {
    if d.source_line.is_empty() {
        if let Some(line) = source.line_text(d.location.line) {
            d.source_line = line.to_string();
        }
    }
    if d.highlight_length == 0 {
        d.highlight_length = 1;
    }
}

fn strip_cop_prefix<'a>(message: &'a str, cop_name: &str) -> &'a str {
    let prefix = format!("{cop_name}: ");
    message.strip_prefix(&prefix).unwrap_or(message)
}

fn style_guide_url(config: &ResolvedConfig, cop_cfg: &CopConfig) -> Option<String> {
    let path = style_guide_path(cop_cfg)?;
    let base = config.style_guide_base_url.as_deref().filter(|s| !s.is_empty());
    Some(resolve_style_guide(base, path))
}

fn style_guide_path(cop_cfg: &CopConfig) -> Option<&str> {
    cop_cfg
        .options
        .get("StyleGuide")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

fn resolve_style_guide(base: Option<&str>, path: &str) -> String {
    base.filter(|_| !is_http_url(path))
        .map(|b| join_url(b, path))
        .unwrap_or_else(|| path.to_string())
}

fn is_http_url(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

fn join_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn run_non_syntax(
    source: &SourceFile,
    tree: &tree_sitter::Tree,
    active: &[ActiveCop<'_>],
    registry: &CopRegistry,
    mode: AutocorrectMode,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<Vec<Correction>>,
) {
    let rest: Vec<ActiveCop<'_>> = active
        .iter()
        .filter(|(c, _, _)| c.name() != "Lint/Syntax")
        .map(|(c, cfg, idx)| (*c, cfg.clone(), *idx))
        .collect();
    run_phases(source, tree, &rest, registry, mode, diagnostics, corrections);
}

fn run_phases(
    source: &SourceFile,
    tree: &tree_sitter::Tree,
    active: &[ActiveCop<'_>],
    registry: &CopRegistry,
    mode: AutocorrectMode,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<Vec<Correction>>,
) {
    let code_map = CodeMap::from_tree(tree.root_node(), source.as_bytes());
    run_line_phase(source, active, mode, diagnostics, corrections);
    run_source_phase(source, tree, &code_map, active, mode, diagnostics, corrections);
    run_node_phase(source, tree, active, registry, mode, diagnostics, corrections);
}

fn filter_directives(source: &SourceFile, ignore_disable: bool, diagnostics: &mut Vec<Diagnostic>) {
    if ignore_disable {
        return;
    }
    let dirs = directives::parse(&String::from_utf8_lossy(source.as_bytes()));
    diagnostics.retain(|d| !dirs.suppresses(&d.cop_name, d.location.line));
}

fn select_active<'a>(
    source: &SourceFile,
    config: &ResolvedConfig,
    registry: &'a CopRegistry,
    filters: &CopFilterSet,
    only: Option<&[String]>,
    except: &[String],
) -> Vec<ActiveCop<'a>> {
    registry
        .cops()
        .iter()
        .enumerate()
        .filter(|(_, cop)| cop_wanted(cop.name(), source.path.as_path(), filters, only, except))
        .map(|(idx, cop)| (&**cop, config.cop_config(cop.name()), idx))
        .collect()
}

fn cop_wanted(
    name: &str,
    path: &std::path::Path,
    filters: &CopFilterSet,
    only: Option<&[String]>,
    except: &[String],
) -> bool {
    if only.is_some_and(|o| !o.iter().any(|n| n == name)) {
        return false;
    }
    if except.iter().any(|e| e == name) {
        return false;
    }
    filters.is_cop_enabled_for_file(name, path)
}

fn corr_bucket(mode: AutocorrectMode) -> Option<Vec<Correction>> {
    (mode != AutocorrectMode::Off).then(Vec::new)
}

fn allow_corr(mode: AutocorrectMode, cop: &dyn Cop) -> bool {
    match mode {
        AutocorrectMode::Off => false,
        AutocorrectMode::Safe => cop.supports_autocorrect() && cop.safe_autocorrect(),
        AutocorrectMode::All => cop.supports_autocorrect(),
    }
}

fn run_line_phase(
    source: &SourceFile,
    active: &[ActiveCop<'_>],
    mode: AutocorrectMode,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<Vec<Correction>>,
) {
    for (cop, cfg, idx) in active {
        let mut corr_buf = allow_corr(mode, *cop).then(Vec::new);
        let before = diagnostics.len();
        cop.check_lines(source, cfg, diagnostics, corr_buf.as_mut());
        finish_cop_pass(diagnostics, before, cfg, &mut corr_buf, *idx, corrections);
    }
}

fn run_source_phase(
    source: &SourceFile,
    tree: &tree_sitter::Tree,
    code_map: &CodeMap,
    active: &[ActiveCop<'_>],
    mode: AutocorrectMode,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<Vec<Correction>>,
) {
    for (cop, cfg, idx) in active {
        let mut corr_buf = allow_corr(mode, *cop).then(Vec::new);
        let before = diagnostics.len();
        cop.check_source(source, tree, code_map, cfg, diagnostics, corr_buf.as_mut());
        finish_cop_pass(diagnostics, before, cfg, &mut corr_buf, *idx, corrections);
    }
}

fn run_node_phase(
    source: &SourceFile,
    tree: &tree_sitter::Tree,
    active: &[ActiveCop<'_>],
    registry: &CopRegistry,
    mode: AutocorrectMode,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<Vec<Correction>>,
) {
    let walker = BatchedWalker::new(
        active.iter().map(|(c, _, _)| *c).collect(),
        active.iter().map(|(_, c, _)| c).collect(),
    );
    let mut node_corr = corr_bucket(mode);
    walker.walk(source, tree.root_node(), diagnostics, node_corr.as_mut());
    stamp_node_corrections(registry, &mut node_corr);
    merge_corrections(corrections, node_corr);
}

fn stamp_node_corrections(registry: &CopRegistry, node_corr: &mut Option<Vec<Correction>>) {
    let Some(buf) = node_corr.as_mut() else {
        return;
    };
    for c in buf {
        if let Some(idx) = registry.index_of(c.cop_name) {
            c.cop_index = idx;
        }
    }
}

fn merge_corrections(into: &mut Option<Vec<Correction>>, from: Option<Vec<Correction>>) {
    if let (Some(all), Some(buf)) = (into.as_mut(), from) {
        all.extend(buf);
    }
}

fn finish_cop_pass(
    diagnostics: &mut [Diagnostic],
    before: usize,
    cfg: &CopConfig,
    corr_buf: &mut Option<Vec<Correction>>,
    idx: usize,
    corrections: &mut Option<Vec<Correction>>,
) {
    if let Some(buf) = corr_buf.as_mut() {
        for c in buf {
            c.cop_index = idx;
        }
    }
    merge_corrections(corrections, corr_buf.take());
    if let Some(sev) = cfg.severity {
        for d in &mut diagnostics[before..] {
            d.severity = sev;
        }
    }
}

fn write_fixes(source: &SourceFile, corrections: Option<Vec<Correction>>) -> Result<()> {
    let Some(corrs) = corrections.filter(|c| !c.is_empty()) else {
        return Ok(());
    };
    let set = CorrectionSet::from_vec(corrs);
    if !set.is_empty() {
        std::fs::write(&source.path, set.apply(source.as_bytes()))?;
    }
    Ok(())
}
