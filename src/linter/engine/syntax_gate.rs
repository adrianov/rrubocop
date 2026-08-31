//! Lint/Syntax first-pass gate (RuboCop: parser failures → Syntax only).

use crate::cli::AutocorrectMode;
use crate::config::ResolvedConfig;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub(crate) type ActiveCop<'a> = (&'a dyn Cop, CopConfig, usize);

/// Returns true when syntax fatals should skip other cops.
pub(crate) fn run(
    source: &SourceFile,
    tree: &tree_sitter::Tree,
    config: &ResolvedConfig,
    active: &[ActiveCop<'_>],
    mode: AutocorrectMode,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<Vec<Correction>>,
) -> bool {
    let code_map = CodeMap::from_tree(tree.root_node(), source.as_bytes());
    match active.iter().find(|(c, _, _)| c.name() == "Lint/Syntax") {
        Some((cop, cfg, idx)) => {
            registered(source, tree, &code_map, *cop, cfg, *idx, mode, diagnostics, corrections)
        }
        None => probe(source, tree, &code_map, config),
    }
}

fn registered(
    source: &SourceFile,
    tree: &tree_sitter::Tree,
    code_map: &CodeMap,
    cop: &dyn Cop,
    cfg: &CopConfig,
    idx: usize,
    mode: AutocorrectMode,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<Vec<Correction>>,
) -> bool {
    let mut corr_buf = allow_corr(mode, cop).then(Vec::new);
    let before = diagnostics.len();
    cop.check_source(source, tree, code_map, cfg, diagnostics, corr_buf.as_mut());
    stamp_and_merge(diagnostics, before, cfg, &mut corr_buf, idx, corrections);
    crate::cop::lint::has_syntax_fatals(diagnostics)
}

fn probe(
    source: &SourceFile,
    tree: &tree_sitter::Tree,
    code_map: &CodeMap,
    config: &ResolvedConfig,
) -> bool {
    let mut out = Vec::new();
    crate::cop::lint::Syntax.check_source(
        source,
        tree,
        code_map,
        &config.cop_config("Lint/Syntax"),
        &mut out,
        None,
    );
    !out.is_empty()
}

fn allow_corr(mode: AutocorrectMode, cop: &dyn Cop) -> bool {
    match mode {
        AutocorrectMode::Off => false,
        AutocorrectMode::Safe => cop.supports_autocorrect() && cop.safe_autocorrect(),
        AutocorrectMode::All => cop.supports_autocorrect(),
    }
}

fn stamp_and_merge(
    diagnostics: &mut [Diagnostic],
    before: usize,
    cfg: &CopConfig,
    corr_buf: &mut Option<Vec<Correction>>,
    idx: usize,
    corrections: &mut Option<Vec<Correction>>,
) {
    if let Some(buf) = corr_buf.as_mut() {
        for c in buf.iter_mut() {
            c.cop_index = idx;
        }
    }
    if let (Some(all), Some(buf)) = (corrections.as_mut(), corr_buf.take()) {
        all.extend(buf);
    }
    if let Some(sev) = cfg.severity {
        for d in &mut diagnostics[before..] {
            d.severity = sev;
        }
    }
}
