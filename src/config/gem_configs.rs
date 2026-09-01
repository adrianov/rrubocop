//! Vendored RuboCop gem configs compiled into the binary.
//!
//! Layout keys: `{gem}/{version}/{relative_path}` (e.g.
//! `rubocop-rails/2.34.3/config/default.yml`). Populated from GitHub via
//! `scripts/fetch_gem_configs.py` and embedded by `build.rs`.
//!
//! Manifest `same_as` maps lockfile versions to a vendored twin when
//! `config/default.yml` (etc.) is byte-identical — used to skip mismatch warnings.

use std::collections::BTreeSet;

include!(concat!(env!("OUT_DIR"), "/gem_configs_embed.rs"));

/// Return file contents for `gem` / `version` / `rel_path`, if embedded.
pub fn file(gem: &str, version: &str, rel_path: &str) -> Option<&'static str> {
    let key = format!("{gem}/{version}/{rel_path}");
    FILES
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, body)| *body)
}

/// Distinct versions embedded for `gem`, sorted ascending.
pub fn versions_for(gem: &str) -> Vec<String> {
    let prefix = format!("{gem}/");
    let mut set = BTreeSet::new();
    for (key, _) in FILES {
        let Some(rest) = key.strip_prefix(&prefix) else {
            continue;
        };
        let Some((ver, _)) = rest.split_once('/') else {
            continue;
        };
        set.insert(ver.to_string());
    }
    set.into_iter().collect()
}

/// Vendored version with identical config YAML for `lock_version`, if recorded.
pub fn same_as(gem: &str, lock_version: &str) -> Option<&'static str> {
    SAME_AS
        .iter()
        .find(|(g, from, _)| *g == gem && *from == lock_version)
        .map(|(_, _, to)| *to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_baseline_rubocop_default() {
        assert!(file("rubocop", "1.84.2", "config/default.yml").is_some());
        assert!(file("rubocop-rails", "2.34.3", "config/default.yml").is_some());
        assert!(file("rubocop-graphql", "1.5.6", "config/default.yml").is_some());
        assert!(file("rubocop-graphql", "1.8.0", "config/default.yml").is_some());
        assert!(file("test-prof", "1.4.4", "config/rubocop-rspec.yml").is_some());
    }

    #[test]
    fn versions_sorted() {
        let v = versions_for("rubocop");
        assert!(v.contains(&"1.77.0".to_string()));
        assert!(v.contains(&"1.79.0".to_string()));
        assert!(v.contains(&"1.84.2".to_string()));
        assert_eq!(v, {
            let mut s = v.clone();
            s.sort();
            s
        });
    }

    #[test]
    fn same_as_rubocop_aliases() {
        assert_eq!(same_as("rubocop", "1.79.2"), Some("1.79.0"));
        assert_eq!(same_as("rubocop", "1.80.2"), Some("1.79.0"));
        assert!(file("rubocop", "1.79.0", "config/default.yml").is_some());
        assert!(same_as("rubocop", "1.81.0").is_none());
    }
}
