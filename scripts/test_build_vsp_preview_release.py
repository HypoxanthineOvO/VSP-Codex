#!/usr/bin/env python3

import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from scripts.build_vsp_preview_release import build_stable_manifest
from scripts.build_vsp_preview_release import default_archive_name
from scripts.build_vsp_preview_release import read_release_version
from scripts.build_vsp_preview_release import read_upstream_tag
from scripts.build_vsp_preview_release import validate_binary_version
from scripts.build_vsp_preview_release import validate_built_package
from scripts.build_vsp_preview_release import validate_release_channel
from scripts.merge_vsp_release_manifests import merge_release_manifests


class ReleaseVersionTest(unittest.TestCase):
    def test_alpha_release_requires_the_explicit_alpha_channel(self) -> None:
        validate_release_channel("0.145.0-vsp.11.0-alpha.1", "alpha")
        validate_release_channel("0.145.0-vsp.9.5", "lab")

        with self.assertRaisesRegex(RuntimeError, "must use.*alpha"):
            validate_release_channel("0.145.0-vsp.11.0-alpha.1", "lab")
        with self.assertRaisesRegex(RuntimeError, "cannot be published"):
            validate_release_channel("0.145.0-vsp.9.5", "alpha")

    def test_matching_workspace_and_package_versions_are_returned(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            workspace_toml, package_json = write_version_files(
                Path(temp_dir), "0.145.0-vsp.9", "0.145.0-vsp.9"
            )

            self.assertEqual(
                read_release_version(workspace_toml, package_json),
                "0.145.0-vsp.9",
            )

    def test_mismatched_workspace_and_package_versions_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            workspace_toml, package_json = write_version_files(
                Path(temp_dir), "0.145.0-vsp.9", "0.140.0-vsp.8.2"
            )

            with self.assertRaisesRegex(RuntimeError, "Release version mismatch"):
                read_release_version(workspace_toml, package_json)

    def test_default_archive_names_are_target_specific(self) -> None:
        self.assertEqual(
            default_archive_name("x86_64-unknown-linux-gnu"),
            "vsp-codex-preview-linux-x64-gnu.tar.gz",
        )
        self.assertEqual(
            default_archive_name("aarch64-apple-darwin"),
            "vsp-codex-preview-macos-arm64.tar.gz",
        )
        self.assertEqual(
            default_archive_name("x86_64-pc-windows-msvc"),
            "vsp-codex-preview-windows-x64-msvc.tar.gz",
        )


class ReleaseArtifactVersionTest(unittest.TestCase):
    def test_binary_version_must_match_release_version(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            binary = write_executable(
                Path(temp_dir) / "vsp-codex",
                "#!/bin/sh\nprintf 'vsp-codex 0.145.0-vsp.9.5\\n'\n",
            )

            validate_binary_version(binary, "0.145.0-vsp.9.5")
            with self.assertRaisesRegex(
                RuntimeError, "Release binary version mismatch"
            ):
                validate_binary_version(binary, "0.145.0-vsp.9.4")

    def test_built_package_metadata_and_binary_must_match(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            package_dir = Path(temp_dir)
            write_executable(
                package_dir / "bin" / "vsp-codex",
                "#!/bin/sh\nprintf 'vsp-codex 0.145.0-vsp.9.5\\n'\n",
            )
            (package_dir / "codex-package.json").write_text(
                json.dumps(
                    {
                        "version": "0.145.0-vsp.9.5",
                        "target": "x86_64-unknown-linux-gnu",
                        "entrypoint": "bin/vsp-codex",
                    }
                ),
                encoding="utf-8",
            )

            validate_built_package(
                package_dir,
                "0.145.0-vsp.9.5",
                "x86_64-unknown-linux-gnu",
            )
            with self.assertRaisesRegex(RuntimeError, "Built package version mismatch"):
                validate_built_package(
                    package_dir,
                    "0.145.0-vsp.9.4",
                    "x86_64-unknown-linux-gnu",
                )


class UpstreamTagTest(unittest.TestCase):
    def test_discovers_latest_upstream_ancestor_and_ignores_vsp_tags(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            git(repo_root, "init", "--quiet")
            git(repo_root, "config", "user.email", "vsp-tests@example.com")
            git(repo_root, "config", "user.name", "VSP Tests")

            commit(repo_root, "baseline.txt", "alpha.2\n", "alpha.2 baseline")
            git(repo_root, "tag", "rust-v0.145.0-alpha.2")
            commit(repo_root, "baseline.txt", "alpha.3\n", "alpha.3 baseline")
            git(repo_root, "tag", "rust-v0.145.0-alpha.3")
            commit(repo_root, "vsp.txt", "vsp.9\n", "VSP changes")
            git(repo_root, "tag", "v0.145.0-vsp.9")

            self.assertEqual(
                read_upstream_tag(repo_root),
                "rust-v0.145.0-alpha.3",
            )


class StableManifestTest(unittest.TestCase):
    def test_new_release_is_first_and_replaces_only_the_same_distribution(self) -> None:
        new_release = release(
            "0.145.0-vsp.9",
            "x86_64-unknown-linux-gnu",
            "new-sha",
            upstream_tag="rust-v0.145.0-alpha.2",
        )
        same_version_other_target = release(
            "0.145.0-vsp.9", "aarch64-apple-darwin", "mac-sha"
        )
        old_release = release("0.140.0-vsp.8.2", "x86_64-unknown-linux-gnu", "old-sha")
        previous = {
            "schema": 1,
            "minimum_updater": "0.134.0-vsp.6",
            "generated_at": "old",
            "releases": [
                release("0.145.0-vsp.9", "x86_64-unknown-linux-gnu", "stale-sha"),
                same_version_other_target,
                old_release,
            ],
        }
        current = {
            "schema": 1,
            "minimum_updater": "0.134.0-vsp.6",
            "upstream_tag": "rust-v0.145.0-alpha.2",
            "generated_at": "new",
            "releases": [new_release],
        }

        self.assertEqual(
            build_stable_manifest(previous, current),
            {
                "schema": 1,
                "minimum_updater": "0.134.0-vsp.6",
                "generated_at": "new",
                "releases": [new_release, same_version_other_target, old_release],
            },
        )

    def test_multi_target_release_replaces_each_matching_distribution(self) -> None:
        linux = release("0.145.0-vsp.12", "x86_64-unknown-linux-gnu", "linux-new")
        macos = release("0.145.0-vsp.12", "aarch64-apple-darwin", "mac-new")
        previous = {
            "releases": [
                release("0.145.0-vsp.12", "x86_64-unknown-linux-gnu", "linux-old"),
                release("0.145.0-vsp.12", "aarch64-apple-darwin", "mac-old"),
                release("0.140.0-vsp.8.2", "x86_64-unknown-linux-gnu", "history"),
            ]
        }
        current = {
            "schema": 1,
            "minimum_updater": "0.134.0-vsp.6",
            "generated_at": "new",
            "releases": [linux, macos],
        }

        self.assertEqual(
            build_stable_manifest(previous, current)["releases"],
            [linux, macos, previous["releases"][2]],
        )


class MergeReleaseManifestsTest(unittest.TestCase):
    def test_merges_linux_macos_and_windows(self) -> None:
        manifests = [
            manifest_for(release("0.145.0-vsp.12", target, f"sha-{target}"), timestamp)
            for target, timestamp in [
                ("x86_64-unknown-linux-gnu", "2026-08-11T01:00:00+00:00"),
                ("aarch64-apple-darwin", "2026-08-11T01:01:00+00:00"),
                ("x86_64-pc-windows-msvc", "2026-08-11T01:02:00+00:00"),
            ]
        ]

        merged = merge_release_manifests(manifests)

        self.assertEqual(
            merged,
            {
                "schema": 1,
                "minimum_updater": "0.134.0-vsp.6",
                "upstream_tag": "rust-v0.145.0-alpha.4",
                "generated_at": "2026-08-11T01:02:00+00:00",
                "releases": [manifest["releases"][0] for manifest in manifests],
            },
        )

    def test_rejects_mismatched_versions_and_duplicate_targets(self) -> None:
        linux = manifest_for(
            release("0.145.0-vsp.12", "x86_64-unknown-linux-gnu", "linux")
        )
        mismatched = manifest_for(
            release("0.145.0-vsp.13", "aarch64-apple-darwin", "mac")
        )
        duplicate = manifest_for(
            release("0.145.0-vsp.12", "x86_64-unknown-linux-gnu", "duplicate")
        )

        with self.assertRaisesRegex(RuntimeError, "metadata does not match"):
            merge_release_manifests([linux, mismatched])
        with self.assertRaisesRegex(RuntimeError, "Duplicate release distribution"):
            merge_release_manifests([linux, duplicate])


def write_version_files(
    root: Path, workspace_version: str, package_version: str
) -> tuple[Path, Path]:
    workspace_toml = root / "Cargo.toml"
    workspace_toml.write_text(
        f'[workspace.package]\nversion = "{workspace_version}"\n',
        encoding="utf-8",
    )
    package_json = root / "package.json"
    package_json.write_text(
        json.dumps({"version": package_version}) + "\n",
        encoding="utf-8",
    )
    return workspace_toml, package_json


def write_executable(path: Path, contents: str) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)
    return path


def release(
    version: str,
    target: str,
    sha256: str,
    *,
    upstream_tag: str | None = None,
) -> dict:
    value = {
        "version": version,
        "channel": "lab",
        "target": target,
        "archive_url": f"https://example.test/{version}/{target}.tar.gz",
        "sha256": sha256,
        "notes_url": f"https://example.test/{version}",
    }
    if upstream_tag is not None:
        value["upstream_tag"] = upstream_tag
    return value


def manifest_for(
    release_value: dict, generated_at: str = "2026-08-11T01:00:00+00:00"
) -> dict:
    release_value = {**release_value, "upstream_tag": "rust-v0.145.0-alpha.4"}
    return {
        "schema": 1,
        "minimum_updater": "0.134.0-vsp.6",
        "upstream_tag": "rust-v0.145.0-alpha.4",
        "generated_at": generated_at,
        "releases": [release_value],
    }


def commit(repo_root: Path, filename: str, contents: str, message: str) -> None:
    (repo_root / filename).write_text(contents, encoding="utf-8")
    git(repo_root, "add", filename)
    git(repo_root, "commit", "--quiet", "-m", message)


def git(repo_root: Path, *args: str) -> None:
    subprocess.run(
        ["git", *args],
        cwd=repo_root,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )


if __name__ == "__main__":
    unittest.main()
