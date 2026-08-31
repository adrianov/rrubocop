//! Read RuboCop YAML; expand ERB via Ruby only when the file contains `<%`.
//!
//! Plain YAML needs no Ruby. ERB configs (Shopify-style `rubocop.yml`) shell out
//! to `ruby` / `bundle exec ruby` — same as RuboCop's ConfigLoader.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use super::gem_path;

const ERB_SCRIPT: &str = concat!(
    "path = ARGV[0]; ",
    "Dir.chdir(File.dirname(path)) { print ERB.new(File.read(path)).result }"
);

/// Read a config file: expand ERB when present (needs Ruby), strip `!ruby/regexp`.
pub(crate) fn load_yaml_text(config_path: &Path, working_dir: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read config {}", config_path.display()))?;
    let expanded = if raw.contains("<%") {
        expand_erb(config_path, working_dir).with_context(|| {
            format!(
                "ERB expansion failed for {} (need `bundle exec ruby` in {})",
                config_path.display(),
                working_dir.display()
            )
        })?
    } else {
        raw
    };
    Ok(expanded.replace("!ruby/regexp ", ""))
}

fn expand_erb(config_path: &Path, working_dir: &Path) -> Result<String> {
    let output = erb_command(working_dir)
        .arg(config_path)
        .output()
        .context("failed to run bundle exec ruby -rerb")?;
    if !output.status.success() {
        anyhow::bail!(
            "ruby ERB exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn erb_command(working_dir: &Path) -> Command {
    let has_gemfile = working_dir.join("Gemfile").exists();
    let mut cmd = if has_gemfile && gem_path::needs_mise_exec(working_dir) {
        let mut c = Command::new("mise");
        c.args(["exec", "--", "bundle", "exec", "ruby", "-rerb", "-e", ERB_SCRIPT]);
        c
    } else if has_gemfile {
        let mut c = Command::new("bundle");
        c.args(["exec", "ruby", "-rerb", "-e", ERB_SCRIPT]);
        c
    } else {
        let mut c = Command::new("ruby");
        c.args(["-rerb", "-e", ERB_SCRIPT]);
        c
    };
    cmd.current_dir(working_dir);
    cmd
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
}
