#!/usr/bin/env python3
"""Compare rrubocop vs nitrocop JSON offenses on a directory of Ruby files.

Usage:
  scripts/cross_test_nitrocop.py [DIR]
  RRUBOCOP=./target/debug/rrubocop NITROCOP=/home/crexus/nitrocop/target/debug/nitrocop \\
    scripts/cross_test_nitrocop.py /tmp/fixtures

Compares offense keys (path basename, line, cop_name). Prints FP/FN summary.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def run_json(cmd: list[str], cwd: Path) -> dict:
    proc = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        check=False,
    )
    # Both tools may exit 1 on offenses
    out = proc.stdout.strip()
    if not out:
        print(f"empty stdout from {cmd[0]}: stderr={proc.stderr[:500]}", file=sys.stderr)
        return {"offenses": []}
    # progress/text noise: take last JSON object line or whole stdout
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        for line in reversed(out.splitlines()):
            line = line.strip()
            if line.startswith("{"):
                return json.loads(line)
        raise


def offense_keys(data: dict, root: Path) -> set[tuple[str, int, str]]:
    keys: set[tuple[str, int, str]] = set()
    for o in data.get("offenses", []):
        path = Path(o["path"])
        try:
            rel = path.resolve().relative_to(root.resolve())
        except Exception:
            rel = Path(path.name)
        keys.add((str(rel), int(o["line"]), o["cop_name"]))
    return keys


def main() -> int:
    fixture = Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    rrubocop = os.environ.get(
        "RRUBOCOP", str(Path(__file__).resolve().parents[1] / "target/debug/rrubocop")
    )
    nitrocop = os.environ.get("NITROCOP", "/home/crexus/nitrocop/target/debug/nitrocop")

    if not Path(rrubocop).exists():
        print(f"missing rrubocop binary: {rrubocop}", file=sys.stderr)
        return 2
    if not Path(nitrocop).exists():
        print(
            f"missing nitrocop binary: {nitrocop} (build nitrocop or set NITROCOP=)",
            file=sys.stderr,
        )
        return 2

    only = os.environ.get(
        "ONLY_COPS",
        "Layout/TrailingWhitespace,Layout/TrailingEmptyLines,Layout/EndOfLine,"
        "Layout/LeadingEmptyLines,Metrics/AbcSize",
    )

    # Prefer explicit file paths so AllCops.Exclude (e.g. tmp/**) does not drop fixtures.
    if fixture.is_dir():
        targets = sorted(p for p in fixture.rglob("*") if p.suffix == ".rb" and p.is_file())
        if not targets:
            print("no .rb files in fixture dir", file=sys.stderr)
            return 2
    else:
        targets = [fixture]

    target_args = [str(p) for p in targets]

    r_cmd = [
        rrubocop,
        "-f",
        "json",
        "--only",
        only,
        "--force-default-config",
        *target_args,
    ]
    n_cmd = [
        nitrocop,
        "-f",
        "json",
        "--only",
        only,
        "--force-default-config",
        *target_args,
    ]

    cwd = fixture if fixture.is_dir() else fixture.parent
    r_data = run_json(r_cmd, cwd)
    n_data = run_json(n_cmd, cwd)

    root = cwd
    r_keys = offense_keys(r_data, root)
    n_keys = offense_keys(n_data, root)

    only_set = set(only.split(","))
    r_keys = {k for k in r_keys if k[2] in only_set}
    n_keys = {k for k in n_keys if k[2] in only_set}

    fp = sorted(r_keys - n_keys)
    fn = sorted(n_keys - r_keys)
    tp = len(r_keys & n_keys)

    print(f"fixture={fixture}")
    print(f"true_positives={tp} rrubocop={len(r_keys)} nitrocop={len(n_keys)}")
    print(f"false_positives={len(fp)} false_negatives={len(fn)}")
    for item in fp[:20]:
        print(f"  FP {item}")
    for item in fn[:20]:
        print(f"  FN {item}")
    if len(fp) > 20 or len(fn) > 20:
        print("  ...")

    return 0 if not fp and not fn else 1


if __name__ == "__main__":
    sys.exit(main())
