"""Repo-level rrubocop vs RuboCop offense-set comparison."""

from __future__ import annotations

import subprocess
from collections import Counter
from pathlib import Path

from cross_test_rubocop_json import scan_keys

_BUNDLE_RUBOCOP = ["bundle", "exec", "rubocop"]


def _version_ok(cmd: list[str], cwd: Path) -> bool:
    try:
        return (
            subprocess.run(
                cmd, cwd=cwd, capture_output=True, text=True, check=False
            ).returncode
            == 0
        )
    except OSError:
        return False


def bundle_rubocop_available(repo: Path) -> bool:
    """True when this repo can run the Gemfile-pinned RuboCop baseline."""
    return _version_ok([*_BUNDLE_RUBOCOP, "--version"], repo)


def _print_side(label: str, items: list[tuple[str, int, str]]) -> None:
    if not items:
        return
    print(
        f"{label} by cop:",
        dict(Counter(cop for _, _, cop in items).most_common(15)),
    )
    for item in items[:10]:
        print(f"  {label} {item}")


def _print_mismatches(
    fp: list[tuple[str, int, str]], fn: list[tuple[str, int, str]]
) -> None:
    _print_side("FP", fp)
    _print_side("FN", fn)
    if len(fp) > 10 or len(fn) > 10:
        print("  ...")


def _print_counts(
    r_keys: set[tuple[str, int, str]],
    rb_keys: set[tuple[str, int, str]],
    fp: list[tuple[str, int, str]],
    fn: list[tuple[str, int, str]],
) -> None:
    print(
        f"true_positives={len(r_keys & rb_keys)} "
        f"rrubocop={len(r_keys)} rubocop={len(rb_keys)}"
    )
    print(f"false_positives={len(fp)} false_negatives={len(fn)}")
    _print_mismatches(fp, fn)


def _report(
    r_keys: set[tuple[str, int, str]], rb_keys: set[tuple[str, int, str]]
) -> int:
    fp = sorted(r_keys - rb_keys)
    fn = sorted(rb_keys - r_keys)
    _print_counts(r_keys, rb_keys, fp, fn)
    return int(bool(fp or fn))


def _load_pair(
    repo: Path, rrubocop: str
) -> tuple[set[tuple[str, int, str]] | None, set[tuple[str, int, str]] | None, str]:
    base = ["-f", "json", "--cache", "false"]
    r_keys, r_err = scan_keys([rrubocop, *base, "."], repo)
    if r_err:
        return None, None, f"rrubocop error: {r_err}"
    rb_keys, rb_err = scan_keys(
        [*_BUNDLE_RUBOCOP, *base, "--disable-pending-cops", "."], repo
    )
    if rb_err:
        return None, None, f"rubocop error: {rb_err}"
    return r_keys, rb_keys, ""


def _finish_compare(repo: Path, rrubocop: str) -> int:
    r_keys, rb_keys, err = _load_pair(repo, rrubocop)
    if err:
        print(err)
        return 2
    return _report(r_keys or set(), rb_keys or set())


def compare_repo(repo: Path, rrubocop: str) -> int:
    print(f"\n{'='*60}\nrepo={repo.name}\n{'='*60}")
    if not bundle_rubocop_available(repo):
        print("skip: bundle exec rubocop unavailable")
        return 0
    return _finish_compare(repo, rrubocop)
