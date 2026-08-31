//! RuboCop `.rubocop.yml` resolution (subset adapted from nitrocop).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde_yml::Value;

use crate::cop::{CopConfig, EnabledState, CopRegistry};
use crate::diagnostic::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewCopsPolicy {
    Enable,
    Disable,
}

#[derive(Debug, Clone)]
struct DepartmentConfig {
    enabled: EnabledState,
    include: Vec<String>,
    exclude: Vec<String>,
}

impl Default for DepartmentConfig {
    fn default() -> Self {
        Self {
            enabled: EnabledState::Unset,
            include: Vec::new(),
            exclude: Vec::new(),
        }
    }
}

pub struct CopFilter {
    enabled: bool,
    include_set: Option<GlobSet>,
    exclude_set: Option<GlobSet>,
}

impl CopFilter {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_match(&self, path: &Path) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(ref inc) = self.include_set
            && !inc.is_match(path)
        {
            return false;
        }
        if let Some(ref exc) = self.exclude_set
            && exc.is_match(path)
        {
            return false;
        }
        true
    }
}

pub struct CopFilterSet {
    filters: HashMap<String, CopFilter>,
    global_exclude: Option<GlobSet>,
}

impl CopFilterSet {
    pub fn build(config: &ResolvedConfig, registry: &CopRegistry) -> Self {
        let mut filters = HashMap::new();
        for cop in registry.cops() {
            let name = cop.name();
            let enabled = config.is_cop_enabled(name, cop.default_enabled());
            let mut include_patterns = config.cop_include(name);
            if include_patterns.is_empty() {
                include_patterns = cop
                    .default_include()
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect();
            }
            let mut exclude_patterns = config.cop_exclude(name);
            exclude_patterns.extend(
                cop.default_exclude()
                    .iter()
                    .map(|s| (*s).to_string()),
            );
            filters.insert(
                name.to_string(),
                CopFilter {
                    enabled,
                    include_set: compile_globs(&include_patterns),
                    exclude_set: compile_globs(&exclude_patterns),
                },
            );
        }
        Self {
            filters,
            global_exclude: compile_globs(&config.global_excludes),
        }
    }

    pub fn is_globally_excluded(&self, path: &Path) -> bool {
        self.global_exclude
            .as_ref()
            .is_some_and(|g| g.is_match(path))
    }

    pub fn filter_for(&self, name: &str) -> Option<&CopFilter> {
        self.filters.get(name)
    }

    pub fn is_cop_enabled_for_file(&self, name: &str, path: &Path) -> bool {
        self.filters
            .get(name)
            .is_some_and(|f| f.is_match(path))
    }
}

fn compile_globs(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        let pat = if p.starts_with('/') {
            p.trim_start_matches('/').to_string()
        } else if (!p.contains('/') && !p.contains('*'))
            || (!p.starts_with("**/") && !p.starts_with('/'))
        {
            format!("**/{p}")
        } else {
            p.clone()
        };
        if let Ok(g) = Glob::new(&pat) {
            builder.add(g);
        }
    }
    builder.build().ok()
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    cop_configs: HashMap<String, CopConfig>,
    department_configs: HashMap<String, DepartmentConfig>,
    pub global_excludes: Vec<String>,
    pub config_dir: Option<PathBuf>,
    new_cops: NewCopsPolicy,
    disabled_by_default: bool,
    pub target_ruby_version: Option<f64>,
}

impl ResolvedConfig {
    pub fn empty() -> Self {
        Self {
            cop_configs: HashMap::new(),
            department_configs: HashMap::new(),
            global_excludes: default_global_excludes(),
            config_dir: None,
            new_cops: NewCopsPolicy::Disable,
            disabled_by_default: false,
            target_ruby_version: None,
        }
    }

    pub fn cop_config(&self, name: &str) -> CopConfig {
        let mut cfg = self
            .cop_configs
            .get(name)
            .cloned()
            .unwrap_or_default();
        if let Some(ver) = self.target_ruby_version {
            cfg.options
                .entry("TargetRubyVersion".into())
                .or_insert(Value::Number(serde_yml::Number::from(ver)));
        }
        cfg
    }

    pub fn is_cop_enabled(&self, name: &str, default_enabled: bool) -> bool {
        if let Some(cfg) = self.cop_configs.get(name) {
            match cfg.enabled {
                EnabledState::True => return true,
                EnabledState::False => return false,
                EnabledState::Pending => {
                    return matches!(self.new_cops, NewCopsPolicy::Enable);
                }
                EnabledState::Unset => {}
            }
        }
        if let Some((dept, _)) = name.split_once('/')
            && let Some(dc) = self.department_configs.get(dept)
        {
            match dc.enabled {
                EnabledState::True => {}
                EnabledState::False => return false,
                EnabledState::Pending => {
                    return matches!(self.new_cops, NewCopsPolicy::Enable);
                }
                EnabledState::Unset => {}
            }
        }
        if self.disabled_by_default {
            false
        } else {
            default_enabled
        }
    }

    pub fn cop_include(&self, name: &str) -> Vec<String> {
        self.cop_configs
            .get(name)
            .map(|c| c.include.clone())
            .unwrap_or_default()
    }

    pub fn cop_exclude(&self, name: &str) -> Vec<String> {
        self.cop_configs
            .get(name)
            .map(|c| c.exclude.clone())
            .unwrap_or_default()
    }
}

fn default_global_excludes() -> Vec<String> {
    vec![
        "node_modules/**/*".into(),
        "tmp/**/*".into(),
        "vendor/**/*".into(),
        ".git/**/*".into(),
    ]
}

pub fn load_config(path: Option<&Path>, target_dir: Option<&Path>) -> Result<ResolvedConfig> {
    let start_dir = target_dir
        .map(|p| {
            if p.is_file() {
                p.parent().unwrap_or(p).to_path_buf()
            } else {
                p.to_path_buf()
            }
        })
        .or_else(|| std::env::current_dir().ok());

    let config_path = match path {
        Some(p) => {
            if p.exists() {
                Some(p.to_path_buf())
            } else {
                return Ok(ResolvedConfig::empty());
            }
        }
        None => start_dir.as_ref().and_then(|d| find_config(d)),
    };

    let Some(config_path) = config_path else {
        let mut cfg = ResolvedConfig::empty();
        cfg.config_dir = start_dir;
        return Ok(cfg);
    };

    let mut visited = HashSet::new();
    let mut resolved = ResolvedConfig::empty();
    resolved.config_dir = config_path.parent().map(|p| p.to_path_buf());
    merge_file(&mut resolved, &config_path, &mut visited)?;
    Ok(resolved)
}

fn find_config(dir: &Path) -> Option<PathBuf> {
    let mut cur = dir.to_path_buf();
    loop {
        for name in [".rubocop.yml", ".rubocop.yaml"] {
            let candidate = cur.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

fn merge_file(
    resolved: &mut ResolvedConfig,
    path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<()> {
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canon.clone()) {
        anyhow::bail!("circular inherit_from involving {}", path.display());
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading config {}", path.display()))?;
    let value: Value = serde_yml::from_str(&text)
        .with_context(|| format!("parsing config {}", path.display()))?;
    let Value::Mapping(map) = value else {
        return Ok(());
    };

    let base = path.parent().unwrap_or(Path::new("."));

    // inherit_from first (parents then child overrides)
    if let Some(inh) = map.get(Value::String("inherit_from".into())) {
        for parent in inherit_list(inh) {
            let parent_path = base.join(&parent);
            if parent_path.is_file() {
                merge_file(resolved, &parent_path, visited)?;
            }
        }
    }

    // inherit_gem: best-effort without requiring Ruby
    if let Some(gems) = map.get(Value::String("inherit_gem".into()))
        && let Value::Mapping(gem_map) = gems
    {
        for (gem_key, files) in gem_map {
            let Some(gem_name) = gem_key.as_str() else {
                continue;
            };
            if let Some(gem_root) = resolve_gem_path(gem_name, base) {
                for file in inherit_list(files) {
                    let gem_cfg = gem_root.join(&file);
                    if gem_cfg.is_file() {
                        merge_file(resolved, &gem_cfg, visited)?;
                    } else {
                        eprintln!(
                            "warning: inherit_gem {gem_name}: missing {}",
                            gem_cfg.display()
                        );
                    }
                }
            } else {
                eprintln!(
                    "warning: inherit_gem '{gem_name}' not found (lint continues with defaults)"
                );
            }
        }
    }

    apply_layer(resolved, &map);
    Ok(())
}

fn inherit_list(v: &Value) -> Vec<String> {
    match v {
        Value::String(s) => vec![s.clone()],
        Value::Sequence(seq) => seq
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn resolve_gem_path(gem_name: &str, from: &Path) -> Option<PathBuf> {
    // Optional: bundle info when Ruby/bundler present
    if let Ok(out) = std::process::Command::new("bundle")
        .args(["info", "--path", gem_name])
        .current_dir(from)
        .output()
        && out.status.success()
    {
        let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    // Fallback: vendor/bundle and common paths
    for root in [
        from.join("vendor/bundle"),
        from.join("vendor/gems"),
        PathBuf::from("/usr/lib/ruby/gems"),
    ] {
        if !root.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with(gem_name) {
                    return Some(entry.path());
                }
            }
        }
        // bundler path layout: ruby/*/gems/name-*
        if let Ok(walker) = ignore::WalkBuilder::new(&root).max_depth(Some(4)).build().collect::<Result<Vec<_>, _>>() {
            for e in walker {
                let p = e.path();
                if p.is_dir() {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name.starts_with(&format!("{gem_name}-")) {
                        return Some(p.to_path_buf());
                    }
                }
            }
        }
    }
    None
}

fn apply_layer(resolved: &mut ResolvedConfig, map: &serde_yml::Mapping) {
    if let Some(all) = map.get(Value::String("AllCops".into()))
        && let Value::Mapping(all_map) = all
    {
        if let Some(v) = all_map.get(Value::String("DisabledByDefault".into())) {
            resolved.disabled_by_default = v.as_bool().unwrap_or(false);
        }
        if let Some(v) = all_map.get(Value::String("NewCops".into())) {
            resolved.new_cops = match v.as_str() {
                Some("enable") => NewCopsPolicy::Enable,
                _ => NewCopsPolicy::Disable,
            };
        }
        if let Some(v) = all_map.get(Value::String("TargetRubyVersion".into())) {
            resolved.target_ruby_version = v
                .as_f64()
                .or_else(|| v.as_u64().map(|u| u as f64))
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()));
        }
        if let Some(ex) = all_map.get(Value::String("Exclude".into())) {
            resolved.global_excludes.extend(string_list(ex));
        }
    }

    for (key, val) in map {
        let Some(name) = key.as_str() else {
            continue;
        };
        if name == "inherit_from"
            || name == "inherit_gem"
            || name == "inherit_mode"
            || name == "require"
            || name == "plugins"
            || name == "AllCops"
        {
            continue;
        }
        let Value::Mapping(cop_map) = val else {
            continue;
        };
        if name.contains('/') {
            let cfg = parse_cop_config(cop_map);
            resolved.cop_configs.insert(name.to_string(), cfg);
        } else {
            // Department-level
            let mut dept = resolved
                .department_configs
                .remove(name)
                .unwrap_or_default();
            if let Some(en) = cop_map.get(Value::String("Enabled".into())) {
                dept.enabled = parse_enabled(en);
            }
            if let Some(inc) = cop_map.get(Value::String("Include".into())) {
                dept.include = string_list(inc);
            }
            if let Some(exc) = cop_map.get(Value::String("Exclude".into())) {
                dept.exclude = string_list(exc);
            }
            resolved.department_configs.insert(name.to_string(), dept);
        }
    }
}

fn parse_cop_config(map: &serde_yml::Mapping) -> CopConfig {
    let mut cfg = CopConfig::default();
    if let Some(en) = map.get(Value::String("Enabled".into())) {
        cfg.enabled = parse_enabled(en);
    }
    if let Some(sev) = map.get(Value::String("Severity".into()))
        && let Some(s) = sev.as_str()
    {
        cfg.severity = Severity::from_str(s);
    }
    if let Some(inc) = map.get(Value::String("Include".into())) {
        cfg.include = string_list(inc);
    }
    if let Some(exc) = map.get(Value::String("Exclude".into())) {
        cfg.exclude = string_list(exc);
    }
    for (k, v) in map {
        let Some(key) = k.as_str() else {
            continue;
        };
        if matches!(key, "Enabled" | "Severity" | "Include" | "Exclude") {
            continue;
        }
        cfg.options.insert(key.to_string(), v.clone());
    }
    cfg
}

fn parse_enabled(v: &Value) -> EnabledState {
    match v {
        Value::Bool(true) => EnabledState::True,
        Value::Bool(false) => EnabledState::False,
        Value::String(s) if s == "pending" => EnabledState::Pending,
        _ => EnabledState::Unset,
    }
}

fn string_list(v: &Value) -> Vec<String> {
    match v {
        Value::String(s) => vec![s.clone()],
        Value::Sequence(seq) => seq
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}
