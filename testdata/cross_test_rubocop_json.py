"""JSON offense extraction for RuboCop / rrubocop parity compares."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


def _parse_stdout(out: str) -> dict:
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        for line in reversed(out.splitlines()):
            if line.strip().startswith("{"):
                return json.loads(line.strip())
        raise


def _json_result(out: str, err: str) -> tuple[dict, str]:
    if not out:
        return {"offenses": []}, f"empty stdout stderr={err[:500]}"
    try:
        return _parse_stdout(out), ""
    except json.JSONDecodeError as e:
        return {"offenses": []}, f"json error: {e} stderr={err[:500]}"


def run_json(cmd: list[str], cwd: Path) -> tuple[dict, str]:
    try:
        proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    except OSError as e:
        return {"offenses": []}, f"exec error: {e}"
    err = proc.stderr.strip()
    if proc.returncode not in (0, 1):
        return {"offenses": []}, f"exit={proc.returncode} stderr={err[:500]}"
    return _json_result(proc.stdout.strip(), err)


def _rel_path(path: Path, root: Path) -> str:
    try:
        return str(path.resolve().relative_to(root.resolve()))
    except Exception:
        return path.name


def _keys_from_top(offenses: list, root: Path) -> set[tuple[str, int, str]]:
    return {
        (_rel_path(Path(o["path"]), root), int(o["line"]), o["cop_name"]) for o in offenses
    }


def _keys_from_files(files: list, root: Path) -> set[tuple[str, int, str]]:
    keys: set[tuple[str, int, str]] = set()
    for f in files:
        rel = _rel_path(Path(f["path"]), root)
        for o in f.get("offenses", []):
            loc = o.get("location", {})
            keys.add((rel, int(loc.get("line", o.get("line", 0))), o["cop_name"]))
    return keys


def offense_keys(data: dict, root: Path) -> set[tuple[str, int, str]]:
    top = data.get("offenses", [])
    return _keys_from_top(top, root) if top else _keys_from_files(data.get("files", []), root)


def scan_keys(
    cmd: list[str], repo: Path
) -> tuple[set[tuple[str, int, str]] | None, str]:
    data, err = run_json(cmd, repo)
    return (None, err) if err else (offense_keys(data, repo), "")
