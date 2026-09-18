//! Process-wide `--rubocop-version` override.
//!
//! Set once by the CLI before any config load; [`super::gem_path_version`]
//! consults it when selecting the vendored `rubocop` default config.

use std::sync::RwLock;

use anyhow::Result;

static VERSION_OVERRIDE: RwLock<Option<String>> = RwLock::new(None);

pub(crate) fn set_rubocop_version_override(version: Option<String>) {
    *VERSION_OVERRIDE
        .write()
        .expect("VERSION_OVERRIDE poisoned") = version;
}

pub(crate) fn rubocop_version_override() -> Option<String> {
    VERSION_OVERRIDE
        .read()
        .expect("VERSION_OVERRIDE poisoned")
        .clone()
}

/// CLI-boundary validation for `--rubocop-version`; must fail the run before
/// any config load because vendored-selection errors are swallowed upstream.
pub fn validate_rubocop_version(version: &str) -> Result<()> {
    if is_well_formed_version(version) {
        Ok(())
    } else {
        anyhow::bail!("invalid --rubocop-version '{version}': expected x.y.z")
    }
}

/// `x.y` / `x.y.z` with numeric parts — stricter than `parse_semver`, which
/// coerces non-numeric parts to zero.
pub(crate) fn is_well_formed_version(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    (2..=3).contains(&parts.len())
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_two_or_three_numeric_parts() {
        assert!(is_well_formed_version("1.79.2"));
        assert!(is_well_formed_version("1.91"));
        assert!(!is_well_formed_version("banana"));
        assert!(!is_well_formed_version("1."));
        assert!(!is_well_formed_version("1.79.x"));
        assert!(!is_well_formed_version(""));
    }

    #[test]
    fn validate_reports_expected_shape() {
        assert!(validate_rubocop_version("1.79.2").is_ok());
        let err = validate_rubocop_version("banana").unwrap_err().to_string();
        assert!(err.contains("invalid --rubocop-version 'banana'"), "{err}");
    }
}
