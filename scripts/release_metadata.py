#!/usr/bin/env python3
"""Validate the changelog's top release and emit GitHub Actions metadata."""

from __future__ import annotations

import argparse
import datetime as dt
import html
import re
import subprocess
from pathlib import Path

HEADER = re.compile(r"^## (\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?) - (\d{4}-\d{2}-\d{2})$")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, help="GITHUB_OUTPUT file; defaults to stdout")
    parser.add_argument("--notes", type=Path, help="write the top changelog entry as release notes")
    parser.add_argument("--commit-sha", help="include this commit's metadata in release notes")
    parser.add_argument("--repository", help="GitHub owner/repository for a commit link")
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
    if args.commit_sha:
        if not args.repository or not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", args.repository):
            raise SystemExit("--repository must be in owner/repository form when --commit-sha is used")
        commit = subprocess.check_output(
            ["git", "show", "-s", "--format=%H%x00%an%x00%ae%x00%aI%x00%B", args.commit_sha],
            cwd=root,
            text=True,
        ).rstrip("\n").split("\0", 4)
        if len(commit) != 5:
            raise SystemExit("Could not read commit metadata")
        sha, author, email, committed_at, message = commit
        if not re.fullmatch(r"[0-9a-fA-F]{40,64}", sha):
            raise SystemExit("Git returned an invalid commit SHA")
        quoted_message = "\n".join(f"> {html.escape(line)}" for line in message.strip().splitlines())
        notes += (
            "\n\n---\n\n### Commit\n\n"
            f"- **Commit:** [`{sha[:12]}`](https://github.com/{args.repository}/commit/{sha})\n"
            f"- **Author:** {html.escape(author)} ({html.escape(email)})\n"
            f"- **Date:** {committed_at}\n\n"
            f"{quoted_message or '> (empty commit message)'}\n"
        )
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
