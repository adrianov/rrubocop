#!/usr/bin/env python3
"""Compare rrubocop vs bundle exec rubocop JSON offenses on a Ruby project."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


def _parse_stdout(out: str) -> dict:
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        for line in reversed(out.splitlines()):
            if line.strip().startswith("{"):
                return json.loads(line.strip())
        raise


def run_json(cmd: list[str], cwd: Path) -> tuple[dict, str]:
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    out = proc.stdout.strip()
    err = proc.stderr.strip()
    if proc.returncode not in (0, 1):
        return {"offenses": []}, f"exit={proc.returncode} stderr={err[:500]}"
    if not out:
        return {"offenses": []}, f"empty stdout stderr={err[:500]}"
    try:
        return _parse_stdout(out), ""
    except json.JSONDecodeError as e:
        return {"offenses": []}, f"json error: {e} stderr={err[:500]}"


def offense_keys(data: dict, root: Path) -> set[tuple[str, int, str]]:
    keys = set()
    top = data.get("offenses", [])
    if top:
        for o in top:
            path = Path(o["path"])
            try:
                rel = str(path.resolve().relative_to(root.resolve()))
            except Exception:
                rel = path.name
            keys.add((rel, int(o["line"]), o["cop_name"]))
        return keys
    for f in data.get("files", []):
        path = Path(f["path"])
        try:
            rel = str(path.resolve().relative_to(root.resolve()))
        except Exception:
            rel = path.name
        for o in f.get("offenses", []):
            loc = o.get("location", {})
            line = loc.get("line", o.get("line", 0))
            keys.add((rel, int(line), o["cop_name"]))
    return keys


def run_json(cmd: list[str], cwd: Path) -> tuple[dict, str]:
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    out = proc.stdout.strip()
    err = proc.stderr.strip()
    if proc.returncode not in (0, 1):
        return {"offenses": []}, f"exit={proc.returncode} stderr={err[:500]}"
    if not out:
        return {"offenses": []}, f"empty stdout stderr={err[:500]}"
    try:
        return _parse_stdout(out), ""
    except json.JSONDecodeError as e:
        return {"offenses": []}, f"json error: {e} stderr={err[:500]}"


def rubocop_available(repo: Path) -> bool:
    proc = subprocess.run(
        ["bundle", "exec", "rubocop", "--version"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )
    return proc.returncode == 0


def compare_repo(repo: Path, rrubocop: str) -> int:
    print(f"\n{'='*60}\nrepo={repo.name}\n{'='*60}")
    if not rubocop_available(repo):
        print("skip: bundle exec rubocop unavailable")
        return 0
    base = ["-f", "json", "--cache", "false"]
    r_data, r_err = run_json([rrubocop, *base, "."], repo)
    if r_err:
        print(f"rrubocop error: {r_err}")
        return 2
    rb_data, rb_err = run_json(
        ["bundle", "exec", "rubocop", *base, "--disable-pending-cops", "."], repo
    )
    if rb_err:
        print(f"rubocop error: {rb_err}")
        return 2

    r_keys = offense_keys(r_data, repo)
    rb_keys = offense_keys(rb_data, repo)
    tp = r_keys & rb_keys
    fp = sorted(r_keys - rb_keys)
    fn = sorted(rb_keys - r_keys)
    print(f"true_positives={len(tp)} rrubocop={len(r_keys)} rubocop={len(rb_keys)}")
    print(f"false_positives={len(fp)} false_negatives={len(fn)}")
    # Group by cop
    fp_cops: dict[str, int] = {}
    fn_cops: dict[str, int] = {}
    for _, _, cop in fp:
        fp_cops[cop] = fp_cops.get(cop, 0) + 1
    for _, _, cop in fn:
        fn_cops[cop] = fn_cops.get(cop, 0) + 1
    if fp_cops:
        print("FP by cop:", dict(sorted(fp_cops.items(), key=lambda x: -x[1])[:15]))
    if fn_cops:
        print("FN by cop:", dict(sorted(fn_cops.items(), key=lambda x: -x[1])[:15]))
    for item in fp[:10]:
        print(f"  FP {item}")
    for item in fn[:10]:
        print(f"  FN {item}")
    if len(fp) > 10 or len(fn) > 10:
        print("  ...")
    return 0 if not fp and not fn else 1


def main() -> int:
    repos_dir = Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/ruby-repos")
    root = Path(__file__).resolve().parents[1]
    rrubocop = os.environ.get("RRUBOCOP", str(root / "target/release/rrubocop"))
    if not Path(rrubocop).exists():
        print(f"missing rrubocop: {rrubocop}", file=sys.stderr)
        return 2
    rc = 0
    for repo in sorted(repos_dir.iterdir()):
        if repo.is_dir() and (repo / "Gemfile").exists():
            r = compare_repo(repo, rrubocop)
            if r > rc:
                rc = r
    return rc


if __name__ == "__main__":
    sys.exit(main())
