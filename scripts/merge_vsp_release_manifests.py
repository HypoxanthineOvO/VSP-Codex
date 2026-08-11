#!/usr/bin/env python3
"""Merge per-target VSP preview manifests into one release bundle manifest."""

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT))

from scripts.build_vsp_preview_release import build_stable_manifest
from scripts.build_vsp_preview_release import release_manifest_key


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Merge validated per-target VSP preview release manifests."
    )
    parser.add_argument(
        "--manifest",
        action="append",
        type=Path,
        required=True,
        help="Per-target manifest to merge. Repeat for each target.",
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--stable-manifest",
        type=Path,
        help="Optional historical manifest to update with the merged release.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    manifests = [read_manifest(path) for path in args.manifest]
    merged = merge_release_manifests(manifests)
    write_manifest(args.output, merged)
    print(f"Wrote {args.output}")

    if args.stable_manifest is not None:
        previous = (
            read_manifest(args.stable_manifest)
            if args.stable_manifest.is_file()
            else {}
        )
        write_manifest(
            args.stable_manifest,
            build_stable_manifest(previous, merged),
        )
        print(f"Wrote {args.stable_manifest}")
    return 0


def read_manifest(path: Path) -> dict:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise RuntimeError(f"Release manifest must be an object: {path}")
    return value


def write_manifest(path: Path, manifest: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def merge_release_manifests(manifests: list[dict]) -> dict:
    if not manifests:
        raise RuntimeError("At least one release manifest is required")

    first = manifests[0]
    expected = common_metadata(first)
    releases = []
    release_keys = set()
    for manifest in manifests:
        actual = common_metadata(manifest)
        if actual != expected:
            raise RuntimeError(
                f"Per-target release metadata does not match: expected {expected}, got {actual}"
            )
        entries = manifest.get("releases")
        if not isinstance(entries, list) or not entries:
            raise RuntimeError("Per-target release manifest has no releases")
        release = entries[0]
        key = release_manifest_key(release)
        if key in release_keys:
            raise RuntimeError(f"Duplicate release distribution: {key}")
        release_keys.add(key)
        releases.append(release)

    return {
        "schema": expected[0],
        "minimum_updater": expected[1],
        "upstream_tag": expected[2],
        "generated_at": max(manifest["generated_at"] for manifest in manifests),
        "releases": releases,
    }


def common_metadata(manifest: dict) -> tuple[object, object, object, object, object]:
    entries = manifest.get("releases")
    if not isinstance(entries, list) or not entries or not isinstance(entries[0], dict):
        raise RuntimeError("Per-target release manifest has no releases")
    release = entries[0]
    return (
        manifest.get("schema"),
        manifest.get("minimum_updater"),
        manifest.get("upstream_tag"),
        release.get("version"),
        release.get("channel"),
    )


if __name__ == "__main__":
    raise SystemExit(main())
