# rrubocop

Fast **RuboCop-compatible** Ruby linter written in Rust. Uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/) — **no Ruby runtime required** for linting.

Drop-in goals: read your existing `.rubocop.yml`, emit RuboCop-like text/JSON, support `-a`/`-A` autocorrect, and grow cop coverage over time.

## Install

```sh
cargo install --path .
```

```sh
rrubocop [OPTIONS] [PATH]...
```

## Features

- **Config** — discovers `.rubocop.yml`, `inherit_from`, `AllCops` / per-cop `Enabled` / `Exclude` / `Include` / options; `inherit_gem` best-effort without requiring Ruby
- **Autocorrect** — `-a` (safe) / `-A` (all)
- **Output** — `progress`, `text`, `json`, `github`, `quiet`, `files`, …
- **Directives** — `# rubocop:disable` / `enable`
- **Parser** — tree-sitter-ruby (no Prism / no CRuby)

Reference implementations: [nitrocop](https://github.com/6/nitrocop) (architecture & fixtures), upstream [RuboCop](https://github.com/rubocop/rubocop).

## Built-in cops (initial set)

| Cop | Notes |
|---|---|
| `Layout/TrailingWhitespace` | autocorrect |
| `Layout/TrailingEmptyLines` | autocorrect |
| `Layout/EndOfLine` | autocorrect |
| `Layout/LeadingEmptyLines` | autocorrect |
| `Layout/EmptyLines` | autocorrect |
| `Layout/SpaceAfterComma` | autocorrect |
| `Layout/SpaceBeforeComma` | autocorrect |
| `Lint/UselessAssignment` | |
| `Metrics/AbcSize` | tree-sitter ABC |
| `Style/FrozenStringLiteralComment` | autocorrect |
| `Style/RedundantBegin` | |
| `Style/EmptyLiteral` | autocorrect |
| `Style/AndOr` | autocorrect |
| `Style/Not` | autocorrect |
| `Naming/AsciiIdentifiers` | |
| `Security/Eval` | |

More cops are ported continuously from nitrocop/RuboCop (breadth first).

## Usage

```sh
rrubocop                         # lint .
rrubocop app lib                 # paths
rrubocop -f json .               # JSON offenses
rrubocop -a .                    # safe autocorrect
rrubocop -A .                    # all autocorrect
rrubocop --only Metrics/AbcSize lib
rrubocop --list-cops
rrubocop -L                      # list target files
```

Exit codes: `0` clean, `1` offenses at/above `--fail-level`, `2` error.

## Cross-test vs nitrocop

With a local nitrocop build:

```sh
scripts/cross_test_nitrocop.py /path/to/fixture_dir
```

Compares `(path, line, cop_name)` offense sets.

## License

GPL-3.0-or-later. See [NOTICE](NOTICE) for MIT attribution of nitrocop-derived design/logic.
