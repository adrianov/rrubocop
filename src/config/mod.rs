//! RuboCop-compatible `.rubocop.yml` resolution (ported from nitrocop).
//!
//! Supports `inherit_from`, `inherit_gem`, `require`/`plugins`, `inherit_mode`,
//! nested configs, `DisabledByDefault`, and `NewCops` / `Enabled: pending`.

pub mod gem_path;
mod gem_configs;
mod gem_path_version;

mod types;
mod yaml_read;
mod globutil;
mod filter;
mod filter_path;
mod filter_match;
mod discover;
mod standard_convert;
mod standard;
mod merge;
mod merge_cop;
mod parse;
mod parse_allcops;
mod parse_cop;
mod load_defaults;
mod load_require;
mod load_gems;
mod load_recursive;
mod load_lockfile;
mod load_resolve;
mod load;
mod ruby_ver;
mod resolved;
mod resolved_state;
mod resolved_enabled;
mod resolved_inject;
mod resolved_cop_cfg;
mod resolved_filters;
mod resolved_effective;
mod resolved_fingerprint;

pub use filter::{CopFilter, CopFilterSet};
pub use load::load_config;
pub use resolved::ResolvedConfig;
pub use types::NewCopsPolicy;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use crate::cop::EnabledState;

    fn write_yaml(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
        path
    }

    fn write_config(dir: &Path, content: &str) -> PathBuf {
        write_yaml(dir, ".rubocop.yml", content)
    }

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/config")
    }

    #[test]
    fn inherit_from_single_file() {
        let dir = tempfile::tempdir().unwrap();
        write_yaml(
            dir.path(),
            "base.yml",
            "Layout/LineLength:\n  Max: 100\nStyle/Foo:\n  Enabled: true\n",
        );
        let path = write_yaml(
            dir.path(),
            ".rubocop.yml",
            "inherit_from: base.yml\nLayout/LineLength:\n  Max: 120\n",
        );
        let config = load_config(Some(&path), None, None).unwrap();
        let cc = config.cop_config("Layout/LineLength");
        assert_eq!(cc.options.get("Max").and_then(|v| v.as_u64()), Some(120));
        assert!(config.is_cop_enabled("Style/Foo", Path::new("a.rb"), &[], &[]));
    }

    #[test]
    fn inherit_from_child_overrides_base() {
        let dir = tempfile::tempdir().unwrap();
        write_yaml(dir.path(), "base.yml", "Style/Foo:\n  Enabled: true\n");
        let path = write_yaml(
            dir.path(),
            ".rubocop.yml",
            "inherit_from: base.yml\nStyle/Foo:\n  Enabled: false\n",
        );
        let config = load_config(Some(&path), None, None).unwrap();
        assert!(!config.is_cop_enabled("Style/Foo", Path::new("a.rb"), &[], &[]));
    }

    #[test]
    fn inherit_from_exclude_appends() {
        let dir = tempfile::tempdir().unwrap();
        write_yaml(
            dir.path(),
            "base.yml",
            "Style/Foo:\n  Exclude:\n    - 'vendor/**'\n",
        );
        let path = write_yaml(
            dir.path(),
            ".rubocop.yml",
            "inherit_from: base.yml\nStyle/Foo:\n  Exclude:\n    - 'tmp/**'\n",
        );
        let config = load_config(Some(&path), None, None).unwrap();
        let cc = config.cop_config("Style/Foo");
        assert!(cc.exclude.contains(&"vendor/**".to_string()));
        assert!(cc.exclude.contains(&"tmp/**".to_string()));
    }

    #[test]
    fn inherit_from_include_replaces() {
        let dir = tempfile::tempdir().unwrap();
        write_yaml(
            dir.path(),
            "base.yml",
            "Style/Foo:\n  Include:\n    - '**/*.rb'\n",
        );
        let path = write_yaml(
            dir.path(),
            ".rubocop.yml",
            "inherit_from: base.yml\nStyle/Foo:\n  Include:\n    - 'app/**'\n",
        );
        let config = load_config(Some(&path), None, None).unwrap();
        assert_eq!(
            config.cop_config("Style/Foo").include,
            vec!["app/**".to_string()]
        );
    }

    #[test]
    fn allcops_exclude_merges_by_default() {
        // RuboCop default inherit_mode merges Exclude for AllCops.
        let dir = tempfile::tempdir().unwrap();
        write_yaml(
            dir.path(),
            "parent.yml",
            "AllCops:\n  Exclude:\n    - 'vendor/**/*'\nLayout/TrailingWhitespace:\n  Enabled: true\n",
        );
        let path = write_yaml(
            dir.path(),
            ".rubocop.yml",
            "inherit_from: parent.yml\nAllCops:\n  Exclude:\n    - 'tmp/**/*'\n",
        );
        let config = load_config(Some(&path), None, None).unwrap();
        let excludes = config.global_excludes();
        assert!(
            excludes.iter().any(|e| e == "vendor/**/*"),
            "parent Exclude should remain: {excludes:?}"
        );
        assert!(
            excludes.iter().any(|e| e == "tmp/**/*"),
            "child Exclude should apply: {excludes:?}"
        );
    }

    #[test]
    fn allcops_exclude_override_replaces() {
        // Override replaces inherit_from Exclude only; RuboCop built-in defaults
        // (vendor/**/*, …) remain. Use a custom pattern so defaults don't mask it.
        let dir = tempfile::tempdir().unwrap();
        write_yaml(
            dir.path(),
            "parent.yml",
            "AllCops:\n  Exclude:\n    - 'spec/fixtures/**/*'\n",
        );
        let path = write_yaml(
            dir.path(),
            ".rubocop.yml",
            "inherit_from: parent.yml\ninherit_mode:\n  override:\n    - Exclude\nAllCops:\n  Exclude:\n    - 'coverage/**/*'\n",
        );
        let config = load_config(Some(&path), None, None).unwrap();
        let excludes = config.global_excludes();
        assert!(
            !excludes.iter().any(|e| e == "spec/fixtures/**/*"),
            "inherited custom exclude should be replaced: {excludes:?}"
        );
        assert!(
            excludes.iter().any(|e| e == "coverage/**/*"),
            "local coverage exclude missing: {excludes:?}"
        );
    }

    #[test]
    fn inherit_mode_merge_include() {
        let dir = tempfile::tempdir().unwrap();
        write_yaml(
            dir.path(),
            "base.yml",
            "Style/Foo:\n  Include:\n    - '**/*.rb'\n",
        );
        let path = write_yaml(
            dir.path(),
            ".rubocop.yml",
            "inherit_from: base.yml\ninherit_mode:\n  merge:\n    - Include\nStyle/Foo:\n  Include:\n    - '**/*.rake'\n",
        );
        let config = load_config(Some(&path), None, None).unwrap();
        let cc = config.cop_config("Style/Foo");
        assert!(cc.include.contains(&"**/*.rb".to_string()));
        assert!(cc.include.contains(&"**/*.rake".to_string()));
    }

    #[test]
    fn inherit_mode_override_exclude() {
        let dir = tempfile::tempdir().unwrap();
        write_yaml(
            dir.path(),
            "base.yml",
            "Style/Foo:\n  Exclude:\n    - 'vendor/**'\n",
        );
        let path = write_yaml(
            dir.path(),
            ".rubocop.yml",
            "inherit_from: base.yml\ninherit_mode:\n  override:\n    - Exclude\nStyle/Foo:\n  Exclude:\n    - 'tmp/**'\n",
        );
        let config = load_config(Some(&path), None, None).unwrap();
        let cc = config.cop_config("Style/Foo");
        assert!(!cc.exclude.contains(&"vendor/**".to_string()));
        assert!(cc.exclude.contains(&"tmp/**".to_string()));
    }

    #[test]
    fn disabled_by_default_disables_unset_cops() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_config(
            dir.path(),
            "AllCops:\n  DisabledByDefault: true\nStyle/Foo:\n  Enabled: true\n",
        );
        let config = load_config(Some(&path), None, None).unwrap();
        assert!(config.is_cop_enabled("Style/Foo", Path::new("a.rb"), &[], &[]));
        assert!(!config.is_cop_enabled("Style/Bar", Path::new("a.rb"), &[], &[]));
    }

    #[test]
    fn circular_inherit_from_breaks_cycle() {
        let path = fixtures_dir().join("inherit_from/circular_a.yml");
        let result = load_config(Some(&path), None, None);
        assert!(result.is_ok(), "cycle should be broken: {result:?}");
    }

    #[test]
    fn diamond_dependency_loads() {
        let path = fixtures_dir().join("inherit_from/diamond_root.yml");
        let config = load_config(Some(&path), None, None).unwrap();
        assert!(!config.is_cop_enabled("Style/Foo", Path::new("a.rb"), &[], &[]));
        assert!(config.is_cop_enabled(
            "Style/FrozenStringLiteralComment",
            Path::new("a.rb"),
            &[],
            &[]
        ));
        assert!(config.is_cop_enabled("Style/StringLiterals", Path::new("a.rb"), &[], &[]));
    }

    #[test]
    fn fixture_inherit_from_merges() {
        let path = fixtures_dir().join("inherit_from/child.yml");
        let config = load_config(Some(&path), None, None).unwrap();
        assert_eq!(
            config
                .cop_config("Layout/LineLength")
                .options
                .get("Max")
                .and_then(|v| v.as_u64()),
            Some(120)
        );
        let excludes = config.global_excludes();
        assert!(excludes.contains(&"vendor/**".to_string()));
        assert!(excludes.contains(&"tmp/**".to_string()));
    }

    fn assert_nested_cop_state(config: &ResolvedConfig, dir: &Path) {
        assert!(config.has_dir_overrides());
        assert_eq!(
            config.cop_config("Style/Foo").enabled,
            EnabledState::True
        );
        let nested = config.cop_config_for_file("Style/Foo", &dir.join("spec/a.rb"));
        assert_eq!(nested.enabled, EnabledState::False);
    }

    #[test]
    fn nested_dir_override_disables_cop() {
        let dir = tempfile::tempdir().unwrap();
        write_config(dir.path(), "Style/Foo:\n  Enabled: true\n");
        write_yaml(
            dir.path(),
            "spec/.rubocop.yml",
            "Style/Foo:\n  Enabled: false\n",
        );
        let config = load_config(None, Some(dir.path()), None).unwrap();
        assert_nested_cop_state(&config, dir.path());
    }

    #[test]
    fn cache_fingerprint_changes_with_config() {
        let dir = tempfile::tempdir().unwrap();
        let a = write_config(dir.path(), "Style/Foo:\n  Enabled: true\n");
        let cfg_a = load_config(Some(&a), None, None).unwrap();
        let b = write_config(dir.path(), "Style/Foo:\n  Enabled: false\n");
        let cfg_b = load_config(Some(&b), None, None).unwrap();
        assert_ne!(cfg_a.cache_fingerprint(), cfg_b.cache_fingerprint());
    }

    #[test]
    fn rails_guides_nested_disables_redundant_percent_q() {
        let root = std::path::Path::new("/tmp/parity/rails");
        if !root.join(".rubocop.yml").exists() {
            return;
        }
        let config = load_config(None, Some(root), None).unwrap();
        assert!(
            config.has_dir_overrides(),
            "expected guides/.rubocop.yml override"
        );
        let abs = root.join("guides/test/epub_test.rb");
        let rel = std::path::Path::new("guides/test/epub_test.rb");
        assert!(
            config.disabled_by_dir_override("Style/RedundantPercentQ", &abs),
            "absolute guides path should disable Style/RedundantPercentQ"
        );
        assert!(
            config.disabled_by_dir_override("Style/RedundantPercentQ", rel),
            "relative guides path should disable Style/RedundantPercentQ"
        );
    }
}
