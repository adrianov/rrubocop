# rrubocop

**Rust RuboCop drop-in replacement** — same CLI, config, and offense output, rewritten in Rust for speed. Point it at an existing Ruby project and use it like RuboCop.

One binary — runs even without Ruby on the system, and the same way regardless of rbenv, rvm, or system Ruby. Uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/); **no Ruby runtime required** for linting. Simple ERB in configs (literals, `<% var = … %>` / `<%= var %>`, `ENV[…]` / `ENV.fetch(…)`) expands natively without Ruby; unsupported ERB falls back to `ruby` / `bundle exec ruby`, matching RuboCop.

Drop-in parity: reads your existing `.rubocop.yml`, emits RuboCop-like text/JSON, supports `-a`/`-A` autocorrect, and grows cop coverage over time.

**Baseline** (shown in `--help` / `--version`): rubocop `1.84.2` with rubocop-rails `2.34.3`, rubocop-performance `1.26.1`, rubocop-rspec `3.9.0`, rubocop-rspec_rails `2.32.0`, rubocop-factory_bot `2.28.0` — pinned in `src/resources/baseline.json`.

## Install

```sh
cargo install --path .
```

```sh
rrubocop [OPTIONS] [PATH]...
```

## Features

- **Config** — RuboCop-compatible resolution: walk-up discovery, `inherit_from` / `inherit_gem` / `require`/`plugins`, `inherit_mode` (merge/override), nested `.rubocop.yml` overrides, `DisabledByDefault`, `NewCops` / `Enabled: pending`, plus per-cop `Enabled` / `Exclude` / `Include` / options; simple ERB in YAML expands natively (Ruby only for unsupported tags)
- **Autocorrect** — `-a` (safe) / `-A` (all)
- **Output** — `progress` (marks stream as files finish), `text`, `json`, `github`, `quiet`, `files`, … (TTY color like RuboCop; `--color` / `--no-color`)
- **Directives** — `# rubocop:disable` / `enable`
- **Parser** — tree-sitter-ruby (no Prism / no CRuby)
- **Cache** — content-addressed `cache.redb` under `$RRUBOCOP_CACHE_DIR` or `$XDG_CACHE_HOME/rrubocop` / `~/.cache/rrubocop` (same style as abcop); `--no-cache` disables

Reference implementations: [nitrocop](https://github.com/6/nitrocop) (architecture & fixtures), upstream [RuboCop](https://github.com/rubocop/rubocop).

## Built-in cops

Departments grow breadth-first from common RuboCop plugin sets (nitrocop / gem sources). Notable coverage:

| Department | Notes |
|---|---|
| `Layout/*` | 90 layout cops (alignment, spacing, empty lines, indentation; many with autocorrect) |
| `Lint/*`, `Style/*`, `Naming/*`, `Security/*` | core set |
| `Metrics/*` | `AbcSize`, `BlockNesting`, `CollectionLiteralLength`, `ParameterLists` |
| `Rails/*` | 62 rails cops |
| `Performance/*` | 24 performance cops |
| `RSpec/*` | 104 rspec cops |
| `RSpecRails/HttpStatus` | 1 |
| `FactoryBot/*` | 3 (`AttributeDefinedStatically`, `CreateList`, `FactoryClassName`) |
| `Bundler/*`, `Gemspec/*`, `Rake/*` | gem / DSL departments |
| `GraphQL/*` | 26 rubocop-graphql cops |

Use `rrubocop --list-cops` for the full registered set, and `rrubocop --list-autocorrectable-cops` for cops with `-a`/`-A` support (~170+ with autocorrect so far; Layout nearly complete; Style/Lint/Rails/RSpec growing).

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
rrubocop -F 10                   # stop after 10 offenses (default 50; `-F` = 1; `-F 0` off)
                                 # with `-a`/`-A`, N is non-autocorrectable offenses only
```

Exit codes: `0` clean, `1` offenses at/above `--fail-level`, `2` error.

## Caching

Content-addressed cache under `$RRUBOCOP_CACHE_DIR` (or `$XDG_CACHE_HOME/rrubocop` / `~/.cache/rrubocop`). Keys cover contents, version, rule revision, `--only`/`--except`, config fingerprint, and path. Auto-pruned to 20 000 entries; `--no-cache` disables. Autocorrect runs bypass the cache. Nothing is written inside the project.

## Benchmarks

Cold lint (cache off) on a large Rails codebase (~3.8k Ruby files), same host, both clean (0 offenses):

| Tool | Wall clock | Notes |
|---|---|---|
| `bundle exec rubocop --cache false` | **2m 37s** (`2:36.64`) | ~99% CPU (mostly single-core) |
| `rrubocop --no-cache` | **7.0s** (`7.005`) | ~1040% CPU (parallel) |

≈ **22×** faster wall clock (`156.6s / 7.0s`). RuboCop user time was 155s; rrubocop used 71s of CPU across cores.

## Cross-test vs nitrocop

With a local nitrocop build:

```sh
testdata/cross_test_nitrocop.py /path/to/fixture_dir
```

Compares `(path, line, cop_name)` offense sets.

## License

GPL-3.0-or-later. See [NOTICE](NOTICE) for MIT attribution of nitrocop-derived design/logic.
