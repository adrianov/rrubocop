#!/usr/bin/env python3
# rubocop:disable Metrics/AbcSize, Metrics/ModuleAbcSize
"""Compare rrubocop vs nitrocop JSON offenses on a directory of Ruby files."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

_DEFAULT_ONLY = (
    "Layout/TrailingWhitespace,Layout/TrailingEmptyLines,Layout/EndOfLine,"
    "Layout/LeadingEmptyLines,Metrics/AbcSize"
)


def _parse_stdout(out: str) -> dict:
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        for line in reversed(out.splitlines()):
            if line.strip().startswith("{"):
                return json.loads(line.strip())
        raise


def run_json(cmd: list[str], cwd: Path) -> dict:
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    out = proc.stdout.strip()
    if out:
        return _parse_stdout(out)
    print(f"empty stdout from {cmd[0]}: stderr={proc.stderr[:500]}", file=sys.stderr)
    return {"offenses": []}


def offense_keys(data: dict, root: Path) -> set[tuple[str, int, str]]:
    keys = set()
    for o in data.get("offenses", []):
        path = Path(o["path"])
        try:
            rel = str(path.resolve().relative_to(root.resolve()))
        except Exception:
            rel = path.name
        keys.add((rel, int(o["line"]), o["cop_name"]))
    return keys


def _require_bin(path: str, label: str) -> str | None:
    if Path(path).exists():
        return path
    print(f"missing {label} binary: {path}", file=sys.stderr)
    return None


def _fixture_targets(fixture: Path) -> list[Path] | None:
    if not fixture.is_dir():
        return [fixture]
    targets = sorted(p for p in fixture.rglob("*.rb") if p.is_file())
    if targets:
        return targets
    print("no .rb files in fixture dir", file=sys.stderr)
    return None


def _compare(r_keys: set, n_keys: set, fixture: Path) -> int:
    fp, fn = sorted(r_keys - n_keys), sorted(n_keys - r_keys)
    print(f"fixture={fixture}")
    print(f"true_positives={len(r_keys & n_keys)} rrubocop={len(r_keys)} nitrocop={len(n_keys)}")
    print(f"false_positives={len(fp)} false_negatives={len(fn)}")
    for item in fp[:20]:
        print(f"  FP {item}")
    for item in fn[:20]:
        print(f"  FN {item}")
    if len(fp) > 20 or len(fn) > 20:
        print("  ...")
    return 0 if not fp and not fn else 1


def main() -> int:
    fixture = Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    root = Path(__file__).resolve().parents[1]
    rrubocop = _require_bin(
        os.environ.get("RRUBOCOP", str(root / "target/debug/rrubocop")), "rrubocop"
    )
    nitrocop = _require_bin(
        os.environ.get("NITROCOP", "/home/crexus/nitrocop/target/debug/nitrocop"),
        "nitrocop",
    )
    if not rrubocop or not nitrocop:
        return 2
    targets = _fixture_targets(fixture)
    if targets is None:
        return 2

    only = os.environ.get("ONLY_COPS", _DEFAULT_ONLY)
    only_set = set(only.split(","))
    args = [str(p) for p in targets]
    cwd = fixture if fixture.is_dir() else fixture.parent
    base = ["-f", "json", "--only", only, "--force-default-config", *args]
    r_keys = {k for k in offense_keys(run_json([rrubocop, *base], cwd), cwd) if k[2] in only_set}
    n_keys = {k for k in offense_keys(run_json([nitrocop, *base], cwd), cwd) if k[2] in only_set}
    return _compare(r_keys, n_keys, fixture)


if __name__ == "__main__":
    sys.exit(main())
# rubocop:enable Metrics/AbcSize, Metrics/ModuleAbcSize
