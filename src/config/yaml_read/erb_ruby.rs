//! Ruby / bundle ERB fallback for config YAML.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::config::gem_path;

const ERB_SCRIPT: &str = concat!(
    "path = ARGV[0]; ",
    "Dir.chdir(File.dirname(path)) { print ERB.new(File.read(path)).result }"
);

pub(super) fn expand_erb(config_path: &Path, working_dir: &Path) -> Result<String> {
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

pub(super) fn erb_command(working_dir: &Path) -> Command {
    let has_gemfile = working_dir.join("Gemfile").exists();
    let mut cmd = ruby_erb_cmd(has_gemfile, working_dir);
    cmd.current_dir(working_dir);
    cmd
}

fn ruby_erb_cmd(has_gemfile: bool, working_dir: &Path) -> Command {
    if has_gemfile && gem_path::needs_mise_exec(working_dir) {
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
    }
}

#[cfg(test)]
pub(super) fn ruby_expand(raw: &str) -> Option<String> {
    Command::new("ruby").arg("-e").arg("").output().ok()?;
    let dir = tempfile::tempdir().ok()?;
    let path = write_temp_config(dir.path(), raw)?;
    run_erb_file(&path, dir.path())
}

#[cfg(test)]
fn write_temp_config(dir: &Path, raw: &str) -> Option<std::path::PathBuf> {
    use std::io::Write;
    let path = dir.join("config.yml");
    std::fs::File::create(&path)
        .ok()?
        .write_all(raw.as_bytes())
        .ok()?;
    Some(path)
}

#[cfg(test)]
fn run_erb_file(path: &Path, working_dir: &Path) -> Option<String> {
    let out = erb_command(working_dir).arg(path).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}
