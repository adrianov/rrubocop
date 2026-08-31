//! `.standard.yml` conversion and standard-family gem config paths.

pub(crate) use super::standard_convert::convert_standard_yml;

/// Mapping from plugin department names to the gem that provides them.
/// Used to register departments from requested gems even when gem resolution fails.
/// Includes standard-family wrapper gems that wrap rubocop plugin gems.
pub(crate) const PLUGIN_GEM_DEPARTMENTS: &[(&str, &str)] = &[
    ("Rails", "rubocop-rails"),
    ("Migration", "rubocop-rails"),
    ("RSpec", "rubocop-rspec"),
    ("RSpecRails", "rubocop-rspec_rails"),
    ("FactoryBot", "rubocop-factory_bot"),
    ("Capybara", "rubocop-capybara"),
    ("Rake", "rubocop-rake"),
    ("Performance", "rubocop-performance"),
    // standard-family wrapper gems
    ("Rails", "standard-rails"),
    ("Migration", "standard-rails"),
    ("Performance", "standard-performance"),
];

/// Upper-bound thresholds → config path. First `ruby_version < bound` wins.
const STANDARD_VERSION_PATHS: &[(f64, &str)] = &[
    (1.9, "config/ruby-1.8.yml"),
    (2.0, "config/ruby-1.9.yml"),
    (2.1, "config/ruby-2.0.yml"),
    (2.2, "config/ruby-2.1.yml"),
    (2.3, "config/ruby-2.2.yml"),
    (2.4, "config/ruby-2.3.yml"),
    (2.5, "config/ruby-2.4.yml"),
    (2.6, "config/ruby-2.5.yml"),
    (2.7, "config/ruby-2.6.yml"),
    (3.0, "config/ruby-2.7.yml"),
    (3.1, "config/ruby-3.0.yml"),
    (3.2, "config/ruby-3.1.yml"),
    (3.3, "config/ruby-3.2.yml"),
    (3.4, "config/ruby-3.3.yml"),
];

const STANDARD_PERF_VERSION_PATHS: &[(f64, &str)] = &[
    (1.9, "config/ruby-1.8.yml"),
    (2.0, "config/ruby-1.9.yml"),
    (2.1, "config/ruby-2.0.yml"),
    (2.2, "config/ruby-2.1.yml"),
    (2.3, "config/ruby-2.2.yml"),
];

fn version_config_path(
    ruby_version: f64,
    table: &'static [(f64, &'static str)],
    fallback: &'static str,
) -> &'static str {
    table
        .iter()
        .find(|(bound, _)| ruby_version < *bound)
        .map(|(_, path)| *path)
        .unwrap_or(fallback)
}

/// Select config file for the `standard` gem based on target ruby version.
/// Mirrors Standard::Base::Plugin — each ruby-X.Y.yml inherits from
/// the next version up, chaining back to base.yml.
pub(crate) fn standard_version_config(ruby_version: f64) -> &'static str {
    version_config_path(ruby_version, STANDARD_VERSION_PATHS, "config/base.yml")
}

/// Select config file for the `standard-performance` gem based on target ruby version.
/// Mirrors Standard::Performance::DeterminesYamlPath.
pub(crate) fn standard_perf_version_config(ruby_version: f64) -> &'static str {
    version_config_path(ruby_version, STANDARD_PERF_VERSION_PATHS, "config/base.yml")
}

/// Map a standard-family gem name to its config file path.
/// Returns None if the gem is not a recognized standard-family gem.
pub(crate) fn standard_gem_config_path(gem_name: &str, ruby_version: Option<f64>) -> Option<&'static str> {
    match gem_name {
        "standard" => Some(standard_version_config(ruby_version.unwrap_or(3.4))),
        "standard-performance" => Some(standard_perf_version_config(ruby_version.unwrap_or(3.4))),
        "standard-rails" | "standard-custom" => Some("config/base.yml"),
        _ => None,
    }
}

/// Returns true if the department belongs to a RuboCop plugin gem and should
/// only run when the corresponding gem is loaded via `require:` or `plugins:`.
///
/// Core departments (Layout, Lint, Style, Metrics, Naming, Security, Bundler,
/// Gemspec) are always available. Plugin departments need their gem loaded.
pub(crate) fn is_plugin_department(dept: &str) -> bool {
    PLUGIN_GEM_DEPARTMENTS.iter().any(|(d, _)| *d == dept)
}
