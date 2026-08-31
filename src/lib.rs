pub mod abc;
pub mod cache;
pub mod cli;
pub mod config;
pub mod cop;
pub mod correction;
pub mod diagnostic;
pub mod formatter;
pub mod fs;
pub mod linter;
pub mod model;
pub mod parse;

use std::io::Read;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use cli::Args;
use config::{CopFilterSet, ResolvedConfig, load_config};
use cop::registry::CopRegistry;
use diagnostic::Diagnostic;
use formatter::create_formatter;
use fs::discover_files;
use linter::{run_linter, should_fail};
use parse::source::SourceFile;

pub fn run() -> Result<ExitCode> {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let args = Args::parse();
    let registry = CopRegistry::default_registry();
    if let Some(code) = list_and_exit(&args, &registry) {
        return Ok(code);
    }

    let config = resolved_config(&args)?;
    if args.stdin.is_some() {
        return lint_stdin(&args, &config, &registry);
    }
    lint_paths(&args, &config, &registry)
}

fn resolved_config(args: &Args) -> Result<ResolvedConfig> {
    if args.force_default_config {
        return Ok(ResolvedConfig::empty());
    }
    let target = args.paths.first().map(|p| p.as_path());
    load_config(args.config.as_deref(), target, None)
}

fn list_and_exit(args: &Args, registry: &CopRegistry) -> Option<ExitCode> {
    if args.list_cops {
        for name in registry.names() {
            println!("{name}");
        }
        return Some(ExitCode::SUCCESS);
    }
    if args.list_autocorrectable_cops {
        for cop in registry.cops() {
            if cop.supports_autocorrect() {
                println!("{}", cop.name());
            }
        }
        return Some(ExitCode::SUCCESS);
    }
    None
}

fn exit_from_diags(diags: &[Diagnostic], fail_level: &str) -> ExitCode {
    if should_fail(diags, fail_level) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn lint_paths(
    args: &Args,
    config: &ResolvedConfig,
    registry: &CopRegistry,
) -> Result<ExitCode> {
    let discovered = discover_files(&args.paths, config)?;
    if args.list_target_files {
        for f in &discovered.files {
            println!("{}", f.display());
        }
        return Ok(ExitCode::SUCCESS);
    }
    let result = run_linter(args, config, registry, &discovered)?;
    create_formatter(&args.format).print(&result.diagnostics, &result.files);
    Ok(exit_from_diags(&result.diagnostics, &args.fail_level))
}

fn lint_stdin(args: &Args, config: &ResolvedConfig, registry: &CopRegistry) -> Result<ExitCode> {
    let stdin_path = args.stdin.as_ref().unwrap();
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf)?;
    let source = SourceFile::from_bytes(stdin_path.clone(), buf);
    let diags = lint_stdin_source(args, config, registry, &source)?;
    create_formatter(&args.format).print(&diags, std::slice::from_ref(stdin_path));
    Ok(exit_from_diags(&diags, &args.fail_level))
}

fn lint_stdin_source(
    args: &Args,
    config: &ResolvedConfig,
    registry: &CopRegistry,
    source: &SourceFile,
) -> Result<Vec<Diagnostic>> {
    let filters = CopFilterSet::build(config, registry);
    let only = (!args.only.is_empty()).then(|| args.only.clone());
    linter::lint_source(
        source,
        config,
        registry,
        &filters,
        only.as_deref(),
        &args.except,
        args.autocorrect_mode(),
        args.ignore_disable_comments,
        false,
    )
}
