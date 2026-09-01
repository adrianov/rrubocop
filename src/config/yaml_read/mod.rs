//! Read RuboCop YAML; expand ERB when present.
//!
//! Plain YAML needs no Ruby. Simple ERB — literals, `<% var = ... %>` /
//! `<%= var %>`, and `ENV[...]` / `ENV.fetch(...)` — is expanded natively in Rust,
//! so common Shopify-style configs work without a Ruby runtime. Anything the native
//! pass does not fully understand falls back to `ruby` / `bundle exec ruby`, matching
//! RuboCop's ConfigLoader.

mod erb_expr;
mod erb_native;
mod erb_ruby;

use std::path::Path;

use anyhow::{Context, Result};

/// Read a config file: expand ERB when present, strip `!ruby/regexp`.
pub(crate) fn load_yaml_text(config_path: &Path, working_dir: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read config {}", config_path.display()))?;
    let expanded = if raw.contains("<%") {
        match erb_native::expand_erb_native(&raw) {
            Some(text) => text,
            None => erb_ruby::expand_erb(config_path, working_dir).with_context(|| {
                format!(
                    "ERB expansion failed for {} (need `bundle exec ruby` in {})",
                    config_path.display(),
                    working_dir.display()
                )
            })?,
        }
    } else {
        raw
    };
    Ok(expanded.replace("!ruby/regexp ", ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_erb() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/config/erb/with_erb.yml")
    }

    fn abc_enabled(text: &str) -> Option<bool> {
        let raw: serde_yml::Value = serde_yml::from_str(text).ok()?;
        raw.get("Metrics/AbcSize")?.get("Enabled")?.as_bool()
    }

    #[test]
    fn erb_expands_and_parses() {
        let path = fixture_erb();
        let text = load_yaml_text(&path, path.parent().unwrap()).unwrap();
        assert!(!text.contains("<%"));
        assert_eq!(abc_enabled(&text), Some(false));
    }

    #[test]
    fn native_expands_fixture_without_ruby() {
        let raw = std::fs::read_to_string(fixture_erb()).unwrap();
        let text = erb_native::expand_erb_native(&raw).expect("simple ERB should expand natively");
        assert!(!text.contains("<%"));
        assert_eq!(abc_enabled(&text), Some(false));
    }

    #[test]
    fn native_handles_supported_forms() {
        unsafe { std::env::set_var("RRUBOCOP_MAX", "123") };
        let raw = concat!(
            "<% style = \"double_quotes\" %>\n",
            "Style/StringLiterals:\n",
            "  EnforcedStyle: <%= style %>\n",
            "Layout/LineLength:\n",
            "  Max: <%= ENV.fetch('RRUBOCOP_MAX', 80) %>\n",
            "Naming/Foo:\n",
            "  Enabled: <%= false %>\n",
            "  Default: <%= ENV.fetch('RRUBOCOP_UNSET', 'yes') %>\n",
        );
        let out = erb_native::expand_erb_native(raw).expect("supported forms expand natively");
        assert!(out.contains("EnforcedStyle: double_quotes"), "{out}");
        assert!(out.contains("Max: 123"), "{out}");
        assert!(out.contains("Enabled: false"), "{out}");
        assert!(out.contains("Default: yes"), "{out}");
    }

    #[test]
    fn native_bails_on_complex() {
        for raw in [
            "<%= 1 + 1 %>",
            "<%= RUBY_VERSION %>",
            "<%= Dir.pwd %>",
            "<% if true %>\nx\n<% end %>",
            "<%% literal %>",
            "<%= 2.7 %>",
        ] {
            assert!(
                erb_native::expand_erb_native(raw).is_none(),
                "should fall back: {raw:?}"
            );
        }
    }

    #[test]
    fn native_matches_ruby() {
        unsafe { std::env::set_var("RRUBOCOP_PARITY", "77") };
        for raw in [
            "<% v = 42 %>\nMax: <%= v %>\n",
            "Max: <%= 1_000 %>\n",
            "S: <%= 'hi' %>\n",
            "S: <%= \"bye\" %>\n",
            "Enabled: <%= true %>\n",
            "Enabled: <%= false %>\n",
            "N: <%= nil %>|end\n",
            "Max: <%= ENV.fetch('RRUBOCOP_PARITY', 80) %>\n",
            "Def: <%= ENV.fetch('RRUBOCOP_UNSET', 'fallback') %>\n",
            "Get: <%= ENV['RRUBOCOP_PARITY'] %>\n",
            "Get: <%= ENV['RRUBOCOP_UNSET'] %>|end\n",
            "<% a = 1; b = 2 %>\n<%= a %>-<%= b %>\n",
            "<%# a comment %>\nStyle/Foo:\n  Enabled: true\n",
        ] {
            let native =
                erb_native::expand_erb_native(raw).unwrap_or_else(|| panic!("native failed: {raw:?}"));
            if let Some(ruby) = erb_ruby::ruby_expand(raw) {
                assert_eq!(native, ruby, "mismatch for {raw:?}");
            }
        }
    }
}
