//! rrubocop — 10x faster RuboCop drop-in (same CLI, configs, and output).

use std::process::ExitCode;

fn main() -> ExitCode {
    match rrubocop::run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(2)
        }
    }
}
