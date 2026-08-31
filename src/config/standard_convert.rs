//! Convert `.standard.yml` into synthetic RuboCop YAML.

use std::path::Path;

use anyhow::Result;

fn map_get<'a>(map: &'a serde_yml::Mapping, key: &str) -> Option<&'a serde_yml::Value> {
    map.get(serde_yml::Value::String(key.into()))
}

fn collect_requires(map: &serde_yml::Mapping) -> Vec<String> {
    let mut requires = vec!["standard".to_string()];
    let Some(plugins) = map_get(map, "plugins").and_then(|v| v.as_sequence()) else {
        return requires;
    };
    for v in plugins {
        if let Some(s) = v.as_str() {
            requires.push(s.to_string());
        }
    }
    requires
}

fn push_require_block(lines: &mut Vec<String>, requires: &[String]) {
    lines.push(format!(
        "require:\n{}",
        requires
            .iter()
            .map(|r| format!("  - {r}"))
            .collect::<Vec<_>>()
            .join("\n")
    ));
}

fn push_target_ruby(all_cops: &mut Vec<String>, map: &serde_yml::Mapping) {
    let Some(rv) = map_get(map, "ruby_version") else {
        return;
    };
    if let Some(f) = rv.as_f64() {
        all_cops.push(format!("  TargetRubyVersion: {f}"));
    } else if let Some(s) = rv.as_str() {
        all_cops.push(format!("  TargetRubyVersion: {s}"));
    }
}

fn default_ignores_disabled(map: &serde_yml::Mapping) -> bool {
    map_get(map, "default_ignores").and_then(|v| v.as_bool()) == Some(false)
}

fn push_ignore_mapping(
    m: &serde_yml::Mapping,
    cop_disables: &mut Vec<(String, String)>,
) {
    for (k, v) in m {
        let (Some(glob), Some(cops)) = (k.as_str(), v.as_sequence()) else {
            continue;
        };
        for cop in cops {
            if let Some(cop_name) = cop.as_str() {
                cop_disables.push((cop_name.to_string(), glob.to_string()));
            }
        }
    }
}

fn collect_ignores(
    map: &serde_yml::Mapping,
) -> (Vec<String>, Vec<(String, String)>) {
    let mut exclude_patterns: Vec<String> = Vec::new();
    if !default_ignores_disabled(map) {
        exclude_patterns.push("bin/*".to_string());
        exclude_patterns.push("db/schema.rb".to_string());
    }
    let mut cop_disables: Vec<(String, String)> = Vec::new();
    let Some(seq) = map_get(map, "ignore").and_then(|v| v.as_sequence()) else {
        return (exclude_patterns, cop_disables);
    };
    for item in seq {
        match item {
            serde_yml::Value::String(pattern) => {
                exclude_patterns.push(pattern.clone());
            }
            serde_yml::Value::Mapping(m) => {
                push_ignore_mapping(m, &mut cop_disables);
            }
            _ => {}
        }
    }
    (exclude_patterns, cop_disables)
}

fn push_exclude_list(all_cops: &mut Vec<String>, patterns: &[String]) {
    if patterns.is_empty() {
        return;
    }
    all_cops.push("  Exclude:".into());
    for p in patterns {
        all_cops.push(format!("    - '{p}'"));
    }
}

fn push_cop_disables(lines: &mut Vec<String>, cop_disables: &[(String, String)]) {
    for (cop_name, glob) in cop_disables {
        if glob == "**/*" {
            lines.push(format!("{cop_name}:\n  Enabled: false"));
        } else {
            lines.push(format!("{cop_name}:\n  Exclude:\n    - '{glob}'"));
        }
    }
}

fn push_extend_config(lines: &mut Vec<String>, map: &serde_yml::Mapping) {
    let Some(seq) = map_get(map, "extend_config").and_then(|v| v.as_sequence()) else {
        return;
    };
    let files: Vec<String> = seq
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    if files.is_empty() {
        return;
    }
    lines.push(format!(
        "inherit_from:\n{}",
        files
            .iter()
            .map(|f| format!("  - {f}"))
            .collect::<Vec<_>>()
            .join("\n")
    ));
}

fn read_standard_doc(standard_path: &Path) -> Result<serde_yml::Value> {
    let content = std::fs::read_to_string(standard_path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", standard_path.display()))?;
    serde_yml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", standard_path.display()))
}

fn build_all_cops_section(
    map: &serde_yml::Mapping,
    exclude_patterns: &[String],
) -> Option<String> {
    let mut all_cops_lines: Vec<String> = Vec::new();
    push_target_ruby(&mut all_cops_lines, map);
    push_exclude_list(&mut all_cops_lines, exclude_patterns);
    if all_cops_lines.is_empty() {
        return None;
    }
    Some(format!("AllCops:\n{}", all_cops_lines.join("\n")))
}

fn emit_converted_yml(map: &serde_yml::Mapping) -> String {
    let mut lines: Vec<String> = Vec::new();
    push_require_block(&mut lines, &collect_requires(map));
    // Standard's DEFAULT_IGNORES: always applied unless `default_ignores: false`.
    // Note: .git/**/*, node_modules/**/*, vendor/**/*, tmp/**/* are already in
    // RuboCop's AllCops.Exclude defaults, so we only need the standard-specific ones.
    let (exclude_patterns, cop_disables) = collect_ignores(map);
    if let Some(section) = build_all_cops_section(map, &exclude_patterns) {
        lines.push(section);
    }
    push_cop_disables(&mut lines, &cop_disables);
    // Standard's ignores append on top of plugin gem configs; merge Exclude arrays.
    if !exclude_patterns.is_empty() {
        lines.push("inherit_mode:\n  merge:\n    - Exclude".to_string());
    }
    push_extend_config(&mut lines, map);
    lines.join("\n\n")
}

pub(crate) fn convert_standard_yml(standard_path: &Path) -> Result<String> {
    let doc = read_standard_doc(standard_path)?;
    let empty_mapping = serde_yml::Mapping::new();
    let map = doc.as_mapping().unwrap_or(&empty_mapping);
    Ok(emit_converted_yml(map))
}
