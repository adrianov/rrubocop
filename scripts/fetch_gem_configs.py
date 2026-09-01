#!/usr/bin/env python3
"""Download RuboCop gem config YAML from GitHub into src/resources/gem_configs/.

Reads src/resources/gem_configs_manifest.json and fetches each listed file
at each version tag (vX.Y.Z). Optional per-gem `same_as` maps lockfile
versions to a vendored twin with identical YAML (no fetch of the alias;
verified byte-equal to the twin). Re-run after editing the manifest.
"""

from __future__ import annotations

import json
import sys
import urllib.error
import urllib.request
from pathlib import Path


def fetch(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": "rrubocop-fetch-gem-configs"})
    with urllib.request.urlopen(req, timeout=60) as resp:
        return resp.read()


def fail_msg(gem: str, version: str, rel: str, err: BaseException) -> None:
    if isinstance(err, urllib.error.HTTPError):
        detail = f"HTTP {err.code}"
    else:
        detail = str(err)
    print(f"FAIL {gem}@{version} {rel}: {detail}", file=sys.stderr)


def write_file(dest: Path, body: bytes, gem: str, version: str, rel: str) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(body)
    print(f"ok   {gem}@{version} {rel} ({len(body)} bytes)")


def fetch_one(out: Path, gem: str, repo: str, version: str, rel: str) -> bool:
    url = f"https://raw.githubusercontent.com/{repo}/v{version}/{rel}"
    try:
        body = fetch(url)
    except Exception as e:  # noqa: BLE001 — report any download failure
        fail_msg(gem, version, rel, e)
        return False
    write_file(out / gem / version / rel, body, gem, version, rel)
    return True


def download_all(manifest: Path, out: Path) -> tuple[int, int]:
    data = json.loads(manifest.read_text())
    ok = fail = 0
    for gem, meta in data["gems"].items():
        for version in meta["versions"]:
            for rel in meta["files"]:
                if fetch_one(out, gem, meta["repo"], version, rel):
                    ok += 1
                else:
                    fail += 1
    return ok, fail


def fail_same_as(gem: str, alias: str, target: str, detail: str) -> None:
    print(f"FAIL {gem} same_as {alias}->{target}: {detail}", file=sys.stderr)


def fetch_alias_body(gem: str, repo: str, alias: str, rel: str) -> bytes | None:
    url = f"https://raw.githubusercontent.com/{repo}/v{alias}/{rel}"
    try:
        return fetch(url)
    except Exception as e:  # noqa: BLE001
        fail_msg(gem, alias, rel, e)
        return None


def bodies_match(
    gem: str, repo: str, alias: str, target: str, rel: str, expected: bytes
) -> bool:
    body = fetch_alias_body(gem, repo, alias, rel)
    if body is None:
        return False
    if body == expected:
        print(f"ok   {gem} same_as {alias} == {target} ({rel})")
        return True
    fail_same_as(gem, alias, target, f"{rel} not byte-identical")
    return False


def check_same_as_file(
    out: Path, gem: str, repo: str, alias: str, target: str, rel: str
) -> bool:
    target_path = out / gem / target / rel
    if not target_path.is_file():
        fail_same_as(gem, alias, target, f"missing {target_path}")
        return False
    return bodies_match(gem, repo, alias, target, rel, target_path.read_bytes())


def verify_alias(out: Path, gem: str, meta: dict, alias: str, target: str) -> int:
    if target not in meta["versions"]:
        fail_same_as(gem, alias, target, "target not in versions")
        return 1
    fail = 0
    for rel in meta["files"]:
        if not check_same_as_file(out, gem, meta["repo"], alias, target, rel):
            fail += 1
    return fail


def verify_same_as(manifest: Path, out: Path) -> int:
    """Ensure each same_as alias target is vendored and YAML matches on GitHub."""
    data = json.loads(manifest.read_text())
    fail = 0
    for gem, meta in data["gems"].items():
        for alias, target in (meta.get("same_as") or {}).items():
            fail += verify_alias(out, gem, meta, alias, target)
    return fail


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    manifest = root / "src/resources/gem_configs_manifest.json"
    out = root / "src/resources/gem_configs"
    ok, fail = download_all(manifest, out)
    fail += verify_same_as(manifest, out)
    print(f"\n{ok} fetched, {fail} failed")
    return 1 if fail else 0


if __name__ == "__main__":
    raise SystemExit(main())
