#!/usr/bin/env python3
"""Validate the changelog's top release and emit GitHub Actions metadata."""

from __future__ import annotations

import argparse
import datetime as dt
import re
from pathlib import Path

HEADER = re.compile(r"^## (\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?) - (\d{4}-\d{2}-\d{2})$")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, help="GITHUB_OUTPUT file; defaults to stdout")
    parser.add_argument("--notes", type=Path, help="write the top changelog entry as release notes")
    args = parser.parse_args()

    root = args.root.resolve()
    cargo_text = (root / "Cargo.toml").read_text(encoding="utf-8")
    cargo_match = re.search(r'(?ms)^\[package\]\s*$.*?^version\s*=\s*"([^"]+)"\s*$', cargo_text)
    if not cargo_match:
        raise SystemExit("Could not read package.version from Cargo.toml")
    cargo_version = cargo_match.group(1)
    lines = (root / "CHANGELOG.md").read_text(encoding="utf-8").splitlines()
    release_index = next((i for i, line in enumerate(lines) if line.startswith("## ")), None)
    if release_index is None:
        raise SystemExit("CHANGELOG.md has no versioned release entry")

    match = HEADER.fullmatch(lines[release_index])
    if not match:
        raise SystemExit("Top changelog heading must use: ## X.Y.Z - YYYY-MM-DD")
    version, release_date = match.groups()
    if version != cargo_version:
        raise SystemExit(f"Changelog version {version} does not match Cargo.toml version {cargo_version}")
    dt.date.fromisoformat(release_date)

    end = next((i for i in range(release_index + 1, len(lines)) if lines[i].startswith("## ")), len(lines))
    notes = "\n".join(lines[release_index + 1 : end]).strip()
    if not notes:
        raise SystemExit("Top changelog release entry has no release notes")
    if args.notes:
        args.notes.write_text(notes + "\n", encoding="utf-8")

    output = args.output.open("a", encoding="utf-8") if args.output else None
    try:
        target = output or __import__("sys").stdout
        print(f"version={version}", file=target)
        print(f"tag=v{version}", file=target)
        print(f"prerelease={'true' if '-' in version else 'false'}", file=target)
    finally:
        if output:
            output.close()


if __name__ == "__main__":
    main()
