#!/usr/bin/env python3
"""Check that the VSP CI policy documents remain present and readable."""

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    for path in (".gitlab-ci.yml", "docs/ci.md", "docs/contributing.md"):
        require(bool(read(path).strip()), f"{path} is empty")
    return 0


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"CI docs check failed: {message}")


if __name__ == "__main__":
    raise SystemExit(main())
