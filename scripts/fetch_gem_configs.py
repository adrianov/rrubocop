#!/usr/bin/env python3
"""Download RuboCop gem config YAML from GitHub into src/resources/gem_configs/.

Reads src/resources/gem_configs_manifest.json and fetches each listed file
at each version tag (vX.Y.Z). Re-run after editing the manifest.
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


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    ok, fail = download_all(
        root / "src/resources/gem_configs_manifest.json",
        root / "src/resources/gem_configs",
    )
    print(f"\n{ok} fetched, {fail} failed")
    return 1 if fail else 0


if __name__ == "__main__":
    raise SystemExit(main())
