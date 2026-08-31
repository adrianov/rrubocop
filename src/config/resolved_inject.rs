//! Option-injection helpers for ResolvedConfig::cop_config.

use std::collections::HashMap;

use serde_yml::Value;

use crate::cop::{CopConfig, EnabledState};

use super::ResolvedConfig;

pub(crate) fn inject_f64(opts: &mut HashMap<String, Value>, key: &str, val: Option<f64>) {
    if let Some(v) = val {
        opts.entry(key.to_string())
            .or_insert_with(|| Value::Number(serde_yml::Number::from(v)));
    }
}

pub(crate) fn inject_bool(opts: &mut HashMap<String, Value>, key: &str, val: bool) {
    opts.entry(key.to_string())
        .or_insert_with(|| Value::Bool(val));
}

pub(crate) fn inject_sibling_str(
    opts: &mut HashMap<String, Value>,
    configs: &HashMap<String, CopConfig>,
    sibling: &str,
    sibling_key: &str,
    dest_key: &str,
    default: &str,
) {
    let style = configs
        .get(sibling)
        .and_then(|cc| cc.options.get(sibling_key))
        .and_then(|v| v.as_str())
        .unwrap_or(default);
    opts.entry(dest_key.to_string())
        .or_insert_with(|| Value::String(style.to_string()));
}

pub(crate) fn sibling_enabled(
    configs: &HashMap<String, CopConfig>,
    sibling: &str,
) -> bool {
    configs
        .get(sibling)
        .map(|cc| !matches!(cc.enabled, EnabledState::False))
        .unwrap_or(true)
}

pub(crate) fn inject_globals(cfg: &ResolvedConfig, config: &mut CopConfig) {
    inject_f64(
        &mut config.options,
        "TargetRubyVersion",
        cfg.target_ruby_version,
    );
    inject_f64(
        &mut config.options,
        "TargetRailsVersion",
        cfg.target_rails_version,
    );
    inject_bool(
        &mut config.options,
        "__RailtiesInLockfile",
        cfg.railties_in_lockfile,
    );
    inject_f64(
        &mut config.options,
        "__RailtiesVersion",
        cfg.railties_version,
    );
}

pub(crate) fn inject_rack_version(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if !matches!(
        name,
        "Rails/HttpStatusNameConsistency" | "RSpecRails/HttpStatusNameConsistency"
    ) {
        return;
    }
    inject_f64(&mut config.options, "__RackVersion", cfg.rack_version);
}

const LINE_LENGTH_COPS: &[&str] = &[
    "Style/IfUnlessModifier",
    "Style/WhileUntilModifier",
    "Style/ConditionalAssignment",
    "Style/GuardClause",
    "Style/SoleNestedConditional",
    "Style/MultilineMethodSignature",
    "Layout/RedundantLineBreak",
];

pub(crate) fn inject_line_length(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if !LINE_LENGTH_COPS.contains(&name) {
        return;
    }
    let ll = cfg.cop_configs.get("Layout/LineLength");
    inject_line_length_max(ll, config);
    inject_line_length_enabled(ll, config);
}

fn inject_line_length_max(ll: Option<&CopConfig>, config: &mut CopConfig) {
    if config.options.contains_key("MaxLineLength") {
        return;
    }
    let max = ll
        .and_then(|cc| cc.options.get("Max"))
        .and_then(|v| v.as_u64())
        .unwrap_or(120);
    config.options.insert(
        "MaxLineLength".to_string(),
        Value::Number(serde_yml::Number::from(max)),
    );
}

fn inject_line_length_enabled(ll: Option<&CopConfig>, config: &mut CopConfig) {
    if config.options.contains_key("LineLengthEnabled") {
        return;
    }
    let enabled = ll
        .map(|cc| !matches!(cc.enabled, EnabledState::False))
        .unwrap_or(true);
    config
        .options
        .insert("LineLengthEnabled".to_string(), Value::Bool(enabled));
}

pub(crate) fn inject_redundant_line_break(
    cfg: &ResolvedConfig,
    name: &str,
    config: &mut CopConfig,
) {
    if name != "Layout/RedundantLineBreak" {
        return;
    }
    if config.options.contains_key("SingleLineBlockChainEnabled") {
        return;
    }
    let enabled = sibling_enabled(&cfg.cop_configs, "Layout/SingleLineBlockChain");
    config.options.insert(
        "SingleLineBlockChainEnabled".to_string(),
        Value::Bool(enabled),
    );
}

const AS_EXTENSION_COPS: &[&str] = &[
    "Lint/DuplicateMethods",
    "Style/ArrayIntersect",
    "Style/CollectionQuerying",
    "Style/RedundantFilterChain",
];

pub(crate) fn inject_active_support(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if !AS_EXTENSION_COPS.contains(&name) {
        return;
    }
    inject_bool(
        &mut config.options,
        "ActiveSupportExtensionsEnabled",
        cfg.active_support_extensions_enabled,
    );
}

pub(crate) fn inject_hash_alignment(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if name != "Layout/HashAlignment" {
        return;
    }
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Layout/ArgumentAlignment",
        "EnforcedStyle",
        "ArgumentAlignmentStyle",
        "with_first_argument",
    );
}

pub(crate) fn inject_first_hash_indent(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if name != "Layout/FirstHashElementIndentation" {
        return;
    }
    let ha = cfg.cop_configs.get("Layout/HashAlignment");
    for (key, default) in [
        ("EnforcedColonStyle", "key"),
        ("EnforcedHashRocketStyle", "key"),
    ] {
        let style = ha
            .and_then(|cc| cc.options.get(key))
            .cloned()
            .unwrap_or_else(|| Value::String(default.to_string()));
        config.options.entry(key.to_string()).or_insert(style);
    }
}

pub(crate) fn inject_end_alignment(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if name != "Layout/ElseAlignment" && name != "Layout/IndentationWidth" {
        return;
    }
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Layout/EndAlignment",
        "EnforcedStyleAlignWith",
        "EndAlignmentStyle",
        "keyword",
    );
}

pub(crate) fn inject_rescue_ensure_alignment(
    cfg: &ResolvedConfig,
    name: &str,
    config: &mut CopConfig,
) {
    if name != "Layout/RescueEnsureAlignment" {
        return;
    }
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Layout/BeginEndAlignment",
        "EnforcedStyleAlignWith",
        "BeginEndAlignmentStyle",
        "begin",
    );
    inject_bool(
        &mut config.options,
        "BeginEndAlignmentEnabled",
        sibling_enabled(&cfg.cop_configs, "Layout/BeginEndAlignment"),
    );
}

pub(crate) fn inject_indentation_width(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if name != "Layout/IndentationWidth" {
        return;
    }
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Layout/IndentationConsistency",
        "EnforcedStyle",
        "IndentationConsistencyStyle",
        "normal",
    );
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Layout/AccessModifierIndentation",
        "EnforcedStyle",
        "AccessModifierIndentationStyle",
        "indent",
    );
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Layout/IndentationStyle",
        "EnforcedStyle",
        "IndentationStyleEnforced",
        "spaces",
    );
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Layout/DefEndAlignment",
        "EnforcedStyleAlignWith",
        "DefEndAlignmentStyle",
        "start_of_line",
    );
}

pub(crate) fn inject_space_after_comma(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if name != "Layout/SpaceAfterComma" {
        return;
    }
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Layout/SpaceInsideHashLiteralBraces",
        "EnforcedStyle",
        "__SpaceInsideHashBracesStyle",
        "space",
    );
}

pub(crate) fn inject_missing_else(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if name != "Style/MissingElse" {
        return;
    }
    inject_bool(
        &mut config.options,
        "UnlessElseEnabled",
        sibling_enabled(&cfg.cop_configs, "Style/UnlessElse"),
    );
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Style/EmptyElse",
        "EnforcedStyle",
        "EmptyElseStyle",
        "both",
    );
}

pub(crate) fn inject_quoted_symbols(cfg: &ResolvedConfig, name: &str, config: &mut CopConfig) {
    if name != "Style/QuotedSymbols" {
        return;
    }
    inject_sibling_str(
        &mut config.options,
        &cfg.cop_configs,
        "Style/StringLiterals",
        "EnforcedStyle",
        "StringLiteralsEnforcedStyle",
        "single_quotes",
    );
}

/// RuboCop CommentConfig seeds `disable_count` with config-disabled cops so
/// `# rubocop:enable Layout/LineLength` is not "extra" when LineLength is off.
pub(crate) fn inject_config_disabled_cops(
    cfg: &ResolvedConfig,
    name: &str,
    config: &mut CopConfig,
) {
    if name != "Lint/RedundantCopEnableDirective" {
        return;
    }
    let disabled: Vec<Value> = cfg
        .cop_configs
        .iter()
        .filter(|(_, c)| matches!(c.enabled, EnabledState::False))
        .map(|(n, _)| Value::String(n.clone()))
        .collect();
    config
        .options
        .entry("ConfigDisabledCops".into())
        .or_insert(Value::Sequence(disabled));
}
