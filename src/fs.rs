use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use ignore::WalkBuilder;

use crate::config::ResolvedConfig;

pub struct DiscoveredFiles {
    pub files: Vec<PathBuf>,
    pub explicit: HashSet<PathBuf>,
}

pub fn discover_files(paths: &[PathBuf], _config: &ResolvedConfig) -> Result<DiscoveredFiles> {
    let mut files = Vec::new();
    let mut explicit = HashSet::new();
    for path in paths {
        collect_path(path, &mut files, &mut explicit)?;
    }
    files.sort();
    files.dedup();
    Ok(DiscoveredFiles { files, explicit })
}

fn collect_path(
    path: &Path,
    files: &mut Vec<PathBuf>,
    explicit: &mut HashSet<PathBuf>,
) -> Result<()> {
    if path.is_file() {
        explicit.insert(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
        files.push(normalize_scan_path(path.to_path_buf()));
        return Ok(());
    }
    if path.is_dir() {
        files.extend(walk_directory(path)?);
        return Ok(());
    }
    anyhow::bail!("path does not exist: {}", path.display());
}

fn walk_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(dir);
    builder
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .follow_links(false);

    let mut files = Vec::new();
    for entry in builder.build() {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_file() && is_ruby_file(path) {
            files.push(normalize_scan_path(path.to_path_buf()));
        }
    }
    Ok(files)
}

/// Drop a leading `./` so offense paths match RuboCop's smart_path style.
fn normalize_scan_path(path: PathBuf) -> PathBuf {
    let Some(s) = path.to_str() else {
        return path;
    };
    match s.strip_prefix("./") {
        Some(rest) => PathBuf::from(rest),
        None => path,
    }
}

const RUBY_EXTENSIONS: &[&str] = &[
    "rb", "arb", "axlsx", "builder", "fcgi", "gemfile", "gemspec", "god", "jb", "jbuilder",
    "mspec", "opal", "pluginspec", "podspec", "rabl", "rake", "rbuild", "rbw", "rbx", "ru",
    "ruby", "schema", "spec", "thor", "watchr",
];

const RUBY_FILENAMES: &[&str] = &[
    ".irbrc",
    ".pryrc",
    ".simplecov",
    "Appraisals",
    "Berksfile",
    "Brewfile",
    "Buildfile",
    "Capfile",
    "Cheffile",
    "Dangerfile",
    "Deliverfile",
    "Fastfile",
    "Gemfile",
    "Guardfile",
    "Jarfile",
    "Mavenfile",
    "Podfile",
    "Puppetfile",
    "Rakefile",
    "rakefile",
    "Schemafile",
    "Snapfile",
    "Steepfile",
    "Thorfile",
    "Vagabondfile",
    "Vagrantfile",
];

fn is_ruby_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str())
        && RUBY_EXTENSIONS.iter().any(|&r| r.eq_ignore_ascii_case(ext))
    {
        return true;
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
        && RUBY_FILENAMES.contains(&name)
    {
        return true;
    }
    false
}
