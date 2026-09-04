#!/usr/bin/env python3
"""Compare rrubocop vs RuboCop JSON offenses on a Ruby project.

Baseline is always `bundle exec rubocop` (project Gemfile). Repos without a
usable bundled RuboCop are skipped.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from cross_test_rubocop_cmp import compare_repo


def _rrubocop_bin() -> str | None:
    path = os.environ.get(
        "RRUBOCOP",
        str(_HERE.parents[0] / "target/release/rrubocop"),
    )
    return path if Path(path).exists() else None


def _scan_repos(repos_dir: Path, rrubocop: str) -> int:
    rc = 0
    for repo in sorted(repos_dir.iterdir()):
        if repo.is_dir() and (repo / "Gemfile").exists():
            rc = max(rc, compare_repo(repo, rrubocop))
    return rc


def main() -> int:
    rrubocop = _rrubocop_bin()
    if rrubocop is None:
        print("missing rrubocop binary", file=sys.stderr)
        return 2
    return _scan_repos(
        Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/ruby-repos"), rrubocop
    )


if __name__ == "__main__":
    sys.exit(main())
