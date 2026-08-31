pub mod abc;
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
use config::{CopFilterSet, load_config};
use cop::registry::CopRegistry;
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

    if args.list_cops {
        for name in registry.names() {
            println!("{name}");
        }
        return Ok(ExitCode::SUCCESS);
    }
    if args.list_autocorrectable_cops {
        for cop in registry.cops() {
            if cop.supports_autocorrect() {
                println!("{}", cop.name());
            }
        }
        return Ok(ExitCode::SUCCESS);
    }

    let target = args.paths.first().map(|p| p.as_path());
    let config = if args.force_default_config {
        config::ResolvedConfig::empty()
    } else {
        load_config(args.config.as_deref(), target)?
    };

    if let Some(ref stdin_path) = args.stdin {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        let source = SourceFile::from_bytes(stdin_path.clone(), buf);
        let filters = CopFilterSet::build(&config, &registry);
        let only = if args.only.is_empty() {
            None
        } else {
            Some(args.only.clone())
        };
        let diags = linter::lint_source(
            &source,
            &config,
            &registry,
            &filters,
            only.as_deref(),
            &args.except,
            args.autocorrect_mode(),
            args.ignore_disable_comments,
            false,
        )?;
        let files = vec![stdin_path.clone()];
        let fmt = create_formatter(&args.format);
        fmt.print(&diags, &files);
        return Ok(if should_fail(&diags, &args.fail_level) {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        });
    }

    let discovered = discover_files(&args.paths, &config)?;
    if args.list_target_files {
        for f in &discovered.files {
            println!("{}", f.display());
        }
        return Ok(ExitCode::SUCCESS);
    }

    let result = run_linter(&args, &config, &registry, &discovered)?;
    let fmt = create_formatter(&args.format);
    fmt.print(&result.diagnostics, &result.files);

    Ok(if should_fail(&result.diagnostics, &args.fail_level) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}
