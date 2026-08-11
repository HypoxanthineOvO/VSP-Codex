#!/usr/bin/env python3
"""Build a VSP Codex preview package, checksum, and release manifest."""

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tarfile
import tempfile
from datetime import datetime
from datetime import timezone
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PACKAGE_BUILDER = REPO_ROOT / "scripts" / "build_codex_package.py"
WORKSPACE_TOML = REPO_ROOT / "codex-rs" / "Cargo.toml"
CODEX_CLI_PACKAGE_JSON = REPO_ROOT / "codex-cli" / "package.json"

DEFAULT_TARGET = "x86_64-unknown-linux-gnu"
DEFAULT_CHANNEL = "lab"
DEFAULT_ARCHIVE_NAMES = {
    "x86_64-unknown-linux-gnu": "vsp-codex-preview-linux-x64-gnu.tar.gz",
    "x86_64-unknown-linux-musl": "vsp-codex-preview-linux-x64-musl.tar.gz",
    "aarch64-unknown-linux-gnu": "vsp-codex-preview-linux-arm64-gnu.tar.gz",
    "aarch64-unknown-linux-musl": "vsp-codex-preview-linux-arm64-musl.tar.gz",
    "x86_64-apple-darwin": "vsp-codex-preview-macos-x64.tar.gz",
    "aarch64-apple-darwin": "vsp-codex-preview-macos-arm64.tar.gz",
    "x86_64-pc-windows-msvc": "vsp-codex-preview-windows-x64-msvc.tar.gz",
    "aarch64-pc-windows-msvc": "vsp-codex-preview-windows-arm64-msvc.tar.gz",
}
DEFAULT_MANIFEST_URL = (
    "https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/"
    ".pipeline/release/vsp-release-manifest.json"
)
DEFAULT_RELEASE_URL_BASE = "https://gitlab.vsplab.cn/heyx/Codex/-/releases"
STABLE_MANIFEST_PATH = REPO_ROOT / ".pipeline" / "release" / "vsp-release-manifest.json"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build the local VSP Codex preview release artifacts.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument("--target", default=DEFAULT_TARGET)
    parser.add_argument("--channel", default=DEFAULT_CHANNEL)
    parser.add_argument("--minimum-updater", default="0.134.0-vsp.6")
    parser.add_argument(
        "--upstream-tag",
        help="Upstream rust-v baseline tag. Defaults to the nearest reachable git tag.",
    )
    parser.add_argument(
        "--release-dir",
        type=Path,
        help="Output directory. Defaults to .pipeline/release/v<workspace-version>.",
    )
    parser.add_argument(
        "--archive-name",
        help="Archive filename. Defaults to the canonical name for --target.",
    )
    parser.add_argument("--manifest-url", default=DEFAULT_MANIFEST_URL)
    parser.add_argument(
        "--release-url-base",
        default=DEFAULT_RELEASE_URL_BASE,
        help="GitLab releases URL base used to build archive and notes URLs.",
    )
    parser.add_argument(
        "--archive-url",
        help="Public archive URL. Defaults to <release-url-base>/v<version>/downloads/<archive-name>.",
    )
    parser.add_argument(
        "--notes-url",
        help="Release notes URL. Defaults to <release-url-base>/v<version>.",
    )
    parser.add_argument(
        "--entrypoint-bin",
        type=Path,
        help="Use an already built vsp-codex binary instead of rebuilding the entrypoint.",
    )
    parser.add_argument(
        "--bwrap-bin",
        type=Path,
        help=(
            "Use a prebuilt bwrap binary. If omitted on Linux and the archive "
            "already exists, the current archive's bwrap is reused."
        ),
    )
    parser.add_argument(
        "--cargo-profile",
        default="release",
        help="Cargo profile used when the package builder builds missing binaries.",
    )
    parser.add_argument(
        "--generated-at",
        help="Manifest timestamp. Defaults to the current local time with timezone.",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Replace existing package/archive artifacts.",
    )
    parser.add_argument(
        "--skip-stable-manifest",
        action="store_true",
        help="Build an isolated candidate without changing the stable release manifest.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    version = read_release_version(WORKSPACE_TOML, CODEX_CLI_PACKAGE_JSON)
    validate_release_channel(version, args.channel)
    if args.entrypoint_bin is not None:
        validate_binary_version(args.entrypoint_bin.resolve(), version)
    upstream_tag = resolve_upstream_tag(args.upstream_tag)
    tag = f"v{version}"
    release_dir = (
        args.release_dir
        if args.release_dir is not None
        else REPO_ROOT / ".pipeline" / "release" / tag
    ).resolve()
    release_dir.mkdir(parents=True, exist_ok=True)
    archive_path = release_dir / (
        args.archive_name or default_archive_name(args.target)
    )
    sha_path = archive_path.with_suffix(archive_path.suffix + ".sha256")
    manifest_path = release_dir / "vsp-release-manifest.json"

    with tempfile.TemporaryDirectory(prefix="vsp-codex-preview-") as temp_dir_name:
        temp_dir = Path(temp_dir_name)
        package_dir = temp_dir / "package"
        bwrap_bin = resolve_bwrap_bin(args.bwrap_bin, archive_path, temp_dir)
        build_cmd = [
            sys.executable,
            str(PACKAGE_BUILDER),
            "--target",
            args.target,
            "--variant",
            "codex",
            "--cargo-profile",
            args.cargo_profile,
            "--package-dir",
            str(package_dir),
            "--archive-output",
            str(archive_path),
            "--vsp-manifest-url",
            args.manifest_url,
            "--vsp-channel",
            args.channel,
            "--vsp-source",
            "preview-release-builder",
        ]
        if args.force:
            build_cmd.append("--force")
        if args.entrypoint_bin is not None:
            build_cmd.extend(["--entrypoint-bin", str(args.entrypoint_bin.resolve())])
        if bwrap_bin is not None:
            build_cmd.extend(["--bwrap-bin", str(bwrap_bin.resolve())])

        subprocess.run(build_cmd, cwd=REPO_ROOT, check=True)
        try:
            validate_built_package(package_dir, version, args.target)
        except Exception:
            archive_path.unlink(missing_ok=True)
            raise

    digest = sha256_file(archive_path)
    sha_path.write_text(f"{digest}  {archive_path.name}\n", encoding="utf-8")

    manifest = {
        "schema": 1,
        "minimum_updater": args.minimum_updater,
        "upstream_tag": upstream_tag,
        "generated_at": args.generated_at or local_timestamp(),
        "releases": [
            {
                "version": version,
                "channel": args.channel,
                "target": args.target,
                "upstream_tag": upstream_tag,
                "archive_url": args.archive_url
                or f"{args.release_url_base.rstrip('/')}/{tag}/downloads/{archive_path.name}",
                "sha256": digest,
                "notes_url": args.notes_url
                or f"{args.release_url_base.rstrip('/')}/{tag}",
            }
        ],
    }

    previous_manifest = read_json_if_exists(manifest_path)
    current_key = release_manifest_key(manifest["releases"][0])
    manifest["releases"].extend(
        release
        for release in previous_manifest.get("releases", [])
        if release_manifest_key(release) != current_key
    )

    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    if not args.skip_stable_manifest:
        stable_manifest = build_stable_manifest(
            read_json_if_exists(STABLE_MANIFEST_PATH), manifest
        )
        STABLE_MANIFEST_PATH.write_text(
            json.dumps(stable_manifest, indent=2) + "\n", encoding="utf-8"
        )

    print(f"Built {archive_path}")
    print(f"Wrote {sha_path}")
    print(f"Wrote {manifest_path}")
    if not args.skip_stable_manifest:
        print(f"Wrote {STABLE_MANIFEST_PATH}")
    return 0


def read_workspace_version(path: Path) -> str:
    in_workspace_package = False
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_workspace_package = stripped == "[workspace.package]"
            continue
        if in_workspace_package and stripped.startswith("version"):
            _, value = stripped.split("=", 1)
            return value.strip().strip('"')
    raise RuntimeError(f"Could not read [workspace.package].version from {path}")


def read_release_version(workspace_toml: Path, package_json: Path) -> str:
    workspace_version = read_workspace_version(workspace_toml)
    package_version = json.loads(package_json.read_text(encoding="utf-8")).get(
        "version"
    )
    if package_version != workspace_version:
        raise RuntimeError(
            "Release version mismatch: "
            f"{workspace_toml} has {workspace_version!r}, "
            f"but {package_json} has {package_version!r}"
        )
    return workspace_version


def default_archive_name(target: str) -> str:
    try:
        return DEFAULT_ARCHIVE_NAMES[target]
    except KeyError as exc:
        supported = ", ".join(sorted(DEFAULT_ARCHIVE_NAMES))
        raise RuntimeError(
            f"Unsupported preview release target {target!r}. Supported targets: {supported}"
        ) from exc


def validate_release_channel(version: str, channel: str) -> None:
    is_alpha = "-alpha." in version
    if is_alpha and channel != "alpha":
        raise RuntimeError(
            f"Alpha release {version!r} must use the explicit 'alpha' channel, got {channel!r}"
        )
    if not is_alpha and channel == "alpha":
        raise RuntimeError(
            f"Non-alpha release {version!r} cannot be published to the alpha channel"
        )


def validate_built_package(
    package_dir: Path, expected_version: str, expected_target: str
) -> None:
    metadata_path = package_dir / "codex-package.json"
    metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    if metadata.get("version") != expected_version:
        raise RuntimeError(
            f"Built package version mismatch: expected {expected_version!r}, "
            f"got {metadata.get('version')!r}"
        )
    if metadata.get("target") != expected_target:
        raise RuntimeError(
            f"Built package target mismatch: expected {expected_target!r}, "
            f"got {metadata.get('target')!r}"
        )

    entrypoint_value = metadata.get("entrypoint")
    if not isinstance(entrypoint_value, str):
        raise RuntimeError("Built package metadata is missing its entrypoint")
    entrypoint = Path(entrypoint_value)
    if entrypoint.is_absolute() or ".." in entrypoint.parts:
        raise RuntimeError(
            f"Built package entrypoint is not relative: {entrypoint_value!r}"
        )
    validate_binary_version(package_dir / entrypoint, expected_version)


def validate_binary_version(binary: Path, expected_version: str) -> None:
    result = subprocess.run(
        [str(binary), "--version"],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or f"exit status {result.returncode}"
        raise RuntimeError(
            f"Could not read release binary version from {binary}: {detail}"
        )
    output = result.stdout.strip()
    actual_version = output.rsplit(maxsplit=1)[-1] if output else ""
    if actual_version != expected_version:
        raise RuntimeError(
            f"Release binary version mismatch: expected {expected_version!r}, "
            f"got {actual_version!r} from {binary}"
        )


def read_json_if_exists(path: Path) -> dict:
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def build_stable_manifest(previous: dict, release_manifest: dict) -> dict:
    current_releases = release_manifest["releases"]
    release_keys = {release_manifest_key(release) for release in current_releases}
    releases = list(current_releases)
    releases.extend(
        candidate
        for candidate in previous.get("releases", [])
        if release_manifest_key(candidate) not in release_keys
    )
    return {
        "schema": release_manifest["schema"],
        "minimum_updater": release_manifest["minimum_updater"],
        "generated_at": release_manifest["generated_at"],
        "releases": releases,
    }


def release_manifest_key(release: dict) -> tuple[str | None, str | None, str | None]:
    return (
        release.get("version"),
        release.get("channel"),
        release.get("target"),
    )


def resolve_upstream_tag(explicit: str | None) -> str:
    if explicit is not None:
        upstream_tag = explicit.strip()
    else:
        upstream_tag = read_upstream_tag(REPO_ROOT)
    if not upstream_tag.startswith("rust-v"):
        raise RuntimeError(
            f"Upstream tag must use the rust-v prefix, got {upstream_tag!r}"
        )
    return upstream_tag


def read_upstream_tag(repo_root: Path) -> str:
    result = subprocess.run(
        ["git", "describe", "--tags", "--match", "rust-v*", "--abbrev=0"],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or "no matching reachable tag"
        raise RuntimeError(f"Could not discover upstream rust-v tag: {detail}")
    return result.stdout.strip()


def resolve_bwrap_bin(
    explicit: Path | None, archive_path: Path, temp_dir: Path
) -> Path | None:
    if explicit is not None:
        return explicit
    if not archive_path.is_file():
        return None

    output = temp_dir / "bwrap"
    with tarfile.open(archive_path, "r:gz") as archive:
        member = next(
            (
                item
                for item in archive.getmembers()
                if item.name.lstrip("./") == "codex-resources/bwrap"
            ),
            None,
        )
        if member is None:
            return None
        source = archive.extractfile(member)
        if source is None:
            return None
        with output.open("wb") as fh:
            shutil.copyfileobj(source, fh)
    output.chmod(0o755)
    return output


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        shutil.copyfileobj(fh, DigestWriter(digest))
    return digest.hexdigest()


class DigestWriter:
    def __init__(self, digest: "hashlib._Hash") -> None:
        self.digest = digest

    def write(self, data: bytes) -> int:
        self.digest.update(data)
        return len(data)


def local_timestamp() -> str:
    return datetime.now(timezone.utc).astimezone().replace(microsecond=0).isoformat()


if __name__ == "__main__":
    raise SystemExit(main())
