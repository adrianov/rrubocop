use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutocorrectMode {
    Off,
    Safe,
    All,
}

#[derive(Parser, Debug)]
#[command(
    name = "rrubocop",
    version = crate::baseline::VERSION,
    about = crate::baseline::ABOUT
)]
pub struct Args {
    /// Files or directories to lint
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(
        short,
        long,
        default_value = "progress",
        value_parser = ["progress", "text", "json", "github", "quiet", "files", "emacs", "simple"]
    )]
    pub format: String,

    /// Run only the specified cops (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub only: Vec<String>,

    /// Exclude the specified cops (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub except: Vec<String>,

    /// Force color output on (RuboCop `--color`)
    #[arg(long, overrides_with = "no_color")]
    pub color: bool,

    /// Force color output off (RuboCop `--no-color`)
    #[arg(long, overrides_with = "color")]
    pub no_color: bool,

    /// Enable debug output
    #[arg(long)]
    pub debug: bool,

    /// List all registered cop names, one per line, then exit
    #[arg(long)]
    pub list_cops: bool,

    /// List cops that support autocorrect, one per line, then exit
    #[arg(long)]
    pub list_autocorrectable_cops: bool,

    /// Read source from stdin, use PATH for display and config matching
    #[arg(long, value_name = "PATH")]
    pub stdin: Option<PathBuf>,

    /// Read cached results (`true`) or lint every file (`false`); writes still update the cache
    #[arg(
        short = 'C',
        long,
        value_name = "FLAG",
        default_value = "true",
        value_parser = ["true", "false"]
    )]
    pub cache: String,

    /// Minimum severity for a non-zero exit code
    #[arg(long, value_name = "SEVERITY", default_value = "convention")]
    pub fail_level: String,

    /// Stop after N offenses (default: 0 = off). Bare `-F` means 1.
    /// With `-a`/`-A`, N counts only non-autocorrectable offenses.
    #[arg(short = 'F', long, value_name = "N", default_value_t = 0)]
    pub fail_fast: u32,

    /// Apply AllCops.Exclude to explicitly-passed files
    #[arg(long)]
    pub force_exclusion: bool,

    /// Print files that would be linted, then exit
    #[arg(short = 'L', long)]
    pub list_target_files: bool,

    /// Display cop names in offense messages (`-D` / AllCops.DisplayCopNames)
    #[arg(short = 'D', long, overrides_with = "no_display_cop_names")]
    pub display_cop_names: bool,

    /// Hide cop names in offense messages
    #[arg(long = "no-display-cop-names", overrides_with = "display_cop_names")]
    pub no_display_cop_names: bool,

    /// Use parallel processing (always on; accepted for RuboCop compat)
    #[arg(short = 'P', long)]
    pub parallel: bool,

    /// Load additional Ruby files (accepted for RuboCop compat; ignored)
    #[arg(short = 'r', long = "require")]
    pub require_libs: Vec<String>,

    /// Ignore all `# rubocop:disable` inline comments
    #[arg(long)]
    pub ignore_disable_comments: bool,

    /// Ignore all config files and use built-in defaults only
    #[arg(long)]
    pub force_default_config: bool,

    /// Autocorrect offenses (safe cops only)
    #[arg(short = 'a', long = "autocorrect")]
    pub autocorrect: bool,

    /// Autocorrect offenses (all cops, including unsafe)
    #[arg(short = 'A', long = "autocorrect-all")]
    pub autocorrect_all: bool,

    /// Start an MCP (Model Context Protocol) server on stdio
    #[arg(long)]
    pub mcp: bool,
}

impl Args {
    /// Parse argv, treating bare `-F` / `--fail-fast` as `-F 1`.
    pub fn parse_cli() -> Self {
        Self::parse_from(normalize_fail_fast(std::env::args_os()))
    }

    pub fn autocorrect_mode(&self) -> AutocorrectMode {
        if self.autocorrect_all {
            AutocorrectMode::All
        } else if self.autocorrect {
            AutocorrectMode::Safe
        } else {
            AutocorrectMode::Off
        }
    }

    /// Whether lint runs may read from the on-disk result cache (`--cache true|false`).
    pub fn cache_read_enabled(&self) -> bool {
        self.cache != "false"
    }

    /// `Some(true/false)` when `--color` / `--no-color`; else auto (TTY).
    pub fn color_force(&self) -> Option<bool> {
        if self.color {
            Some(true)
        } else if self.no_color {
            Some(false)
        } else {
            None
        }
    }

    /// CLI override for AllCops.DisplayCopNames (`-D` / `--no-display-cop-names`).
    pub fn display_cop_names_override(&self) -> Option<bool> {
        if self.display_cop_names {
            Some(true)
        } else if self.no_display_cop_names {
            Some(false)
        } else {
            None
        }
    }
}

fn is_fail_fast_n(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn push_fail_fast_arg(out: &mut Vec<std::ffi::OsString>, next: Option<std::ffi::OsString>) {
    match next {
        Some(n) if is_fail_fast_n(&n.to_string_lossy()) => out.push(n),
        Some(n) => {
            out.push(std::ffi::OsString::from("1"));
            out.push(n);
        }
        None => out.push(std::ffi::OsString::from("1")),
    }
}

/// Insert `1` after bare `-F` / `--fail-fast` so the next path is not eaten.
fn normalize_fail_fast<I>(raw: I) -> Vec<std::ffi::OsString>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut out = Vec::new();
    let mut iter = raw.into_iter();
    if let Some(bin) = iter.next() {
        out.push(bin);
    }
    while let Some(arg) = iter.next() {
        let bare = {
            let s = arg.to_string_lossy();
            s == "-F" || s == "--fail-fast"
        };
        out.push(arg);
        if bare {
            push_fail_fast_arg(&mut out, iter.next());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{normalize_fail_fast, Args};
    use clap::Parser;
    use std::ffi::OsString;

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    fn s(args: Vec<OsString>) -> Vec<String> {
        args.into_iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn bare_f_inserts_one_before_path() {
        assert_eq!(
            s(normalize_fail_fast(os(&["rr", "-F", "app"]))),
            vec!["rr", "-F", "1", "app"]
        );
    }

    #[test]
    fn f_with_number_unchanged() {
        assert_eq!(
            s(normalize_fail_fast(os(&["rr", "-F", "3", "."]))),
            vec!["rr", "-F", "3", "."]
        );
    }

    #[test]
    fn bare_f_at_end() {
        assert_eq!(
            s(normalize_fail_fast(os(&["rr", "-F"]))),
            vec!["rr", "-F", "1"]
        );
    }

    #[test]
    fn cache_flag_defaults_on_and_accepts_false() {
        let on = Args::parse_from(["rr"]);
        assert!(on.cache_read_enabled());
        let off = Args::parse_from(["rr", "--cache", "false"]);
        assert!(!off.cache_read_enabled());
        let short = Args::parse_from(["rr", "-C", "false"]);
        assert!(!short.cache_read_enabled());
    }
}
