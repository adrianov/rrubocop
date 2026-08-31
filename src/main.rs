//! rrubocop — fast RuboCop-compatible Ruby linter (tree-sitter).

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
