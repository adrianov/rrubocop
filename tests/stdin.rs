//! `--stdin` autocorrect: RuboCop prints the fixed source and never writes the file.

use std::fs::File;
use std::path::Path;
use std::process::Command;

/// Stdout of `rrubocop <args> --stdin foo.rb` run in `dir` with `source` as stdin.
fn run_stdin(dir: &Path, source: &str, args: &[&str]) -> String {
    let input = dir.join("input");
    std::fs::write(&input, source).unwrap();
    String::from_utf8(
        Command::new(env!("CARGO_BIN_EXE_rrubocop"))
            .current_dir(dir)
            .args(["--force-default-config", "--cache", "false"])
            .args(args)
            .args(["--stdin", "foo.rb"])
            .stdin(File::open(&input).unwrap())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
}

#[test]
fn stdin_autocorrect_prints_source_and_keeps_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("foo.rb");
    std::fs::write(&file, "on_disk = 1\n").unwrap();

    let out = run_stdin(
        dir.path(),
        "x = 1  \n",
        &["-a", "--only", "Layout/TrailingWhitespace"],
    );

    assert!(out.ends_with("\n====================\nx = 1\n"), "{out}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "on_disk = 1\n");
}

#[test]
fn autocorrect_skips_disabled_lines() {
    let disabled = "x = \"a\" # rubocop:disable Style/StringLiterals\n";

    let out = run_stdin(
        tempfile::tempdir().unwrap().path(),
        &format!("{disabled}y = \"b\"\n"),
        &["-a", "--only", "Style/StringLiterals"],
    );

    assert!(
        out.ends_with(&format!("\n====================\n{disabled}y = 'b'\n")),
        "{out}"
    );
}

#[test]
fn stdin_offenses_sorted_by_column() {
    let out = run_stdin(
        tempfile::tempdir().unwrap().path(),
        "x = \"a\"  \n",
        &[
            "-f",
            "emacs",
            "--only",
            "Style/StringLiterals,Layout/TrailingWhitespace",
        ],
    );

    let order: Vec<_> = out.lines().filter_map(|l| l.split(": ").nth(2)).collect();
    assert_eq!(
        order,
        [
            "[Correctable] Style/StringLiterals",
            "[Correctable] Layout/TrailingWhitespace"
        ],
        "{out}"
    );
}
