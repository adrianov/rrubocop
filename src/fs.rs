use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use ignore::WalkBuilder;

use crate::config::{CopFilterSet, ResolvedConfig};

pub struct DiscoveredFiles {
    pub files: Vec<PathBuf>,
    pub explicit: HashSet<PathBuf>,
}

/// Discover Ruby files under `paths`, applying `AllCops.Exclude` from `config`.
pub fn discover_files(paths: &[PathBuf], config: &ResolvedConfig) -> Result<DiscoveredFiles> {
    discover_files_filtered(paths, &CopFilterSet::for_discover(config), false)
}

/// Discover Ruby files under `paths`, honoring `filters` (same as lint walk).
pub fn discover_files_filtered(
    paths: &[PathBuf],
    filters: &CopFilterSet,
    force_exclusion: bool,
) -> Result<DiscoveredFiles> {
    let mut files = Vec::new();
    let mut explicit = HashSet::new();
    let stop = AtomicBool::new(false);
    discover_emitting(paths, filters, force_exclusion, &stop, |p| {
        files.push(p);
    })?;
    for path in paths {
        if path.is_file() {
            explicit.insert(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
        }
    }
    files.sort();
    files.dedup();
    Ok(DiscoveredFiles { files, explicit })
}

/// Walk `paths` and invoke `emit` for each included Ruby file as soon as it is
/// found. Stops early when `stop` is set (fail-fast). Returns how many paths
/// were emitted.
pub fn discover_emitting(
    paths: &[PathBuf],
    filters: &CopFilterSet,
    force_exclusion: bool,
    stop: &AtomicBool,
    mut emit: impl FnMut(PathBuf),
) -> Result<usize> {
    let mut explicit = HashSet::new();
    let mut count = 0usize;
    let mut seen = HashSet::new();
    for path in paths {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        collect_emitting(
            path,
            filters,
            force_exclusion,
            stop,
            &mut explicit,
            &mut seen,
            &mut count,
            &mut emit,
        )?;
    }
    Ok(count)
}

fn collect_emitting(
    path: &Path,
    filters: &CopFilterSet,
    force_exclusion: bool,
    stop: &AtomicBool,
    explicit: &mut HashSet<PathBuf>,
    seen: &mut HashSet<PathBuf>,
    count: &mut usize,
    emit: &mut impl FnMut(PathBuf),
) -> Result<()> {
    if path.is_file() {
        let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        explicit.insert(canon);
        let norm = normalize_scan_path(path.to_path_buf());
        maybe_emit(norm, true, force_exclusion, filters, seen, count, emit);
        return Ok(());
    }
    if path.is_dir() {
        walk_directory_emitting(path, filters, force_exclusion, stop, seen, count, emit)?;
        return Ok(());
    }
    anyhow::bail!("path does not exist: {}", path.display());
}

fn maybe_emit(
    path: PathBuf,
    is_explicit: bool,
    force_exclusion: bool,
    filters: &CopFilterSet,
    seen: &mut HashSet<PathBuf>,
    count: &mut usize,
    emit: &mut impl FnMut(PathBuf),
) {
    if !seen.insert(path.clone()) {
        return;
    }
    if is_explicit && !force_exclusion {
        *count += 1;
        emit(path);
        return;
    }
    if filters.is_globally_excluded(&path) {
        return;
    }
    *count += 1;
    emit(path);
}

fn walk_directory_emitting(
    dir: &Path,
    filters: &CopFilterSet,
    force_exclusion: bool,
    stop: &AtomicBool,
    seen: &mut HashSet<PathBuf>,
    count: &mut usize,
    emit: &mut impl FnMut(PathBuf),
) -> Result<()> {
    let mut builder = WalkBuilder::new(dir);
    builder
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .follow_links(false);

    for entry in builder.build() {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_file() && is_ruby_file(path) {
            let norm = normalize_scan_path(path.to_path_buf());
            maybe_emit(norm, false, force_exclusion, filters, seen, count, emit);
        }
    }
    Ok(())
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
