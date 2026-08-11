#!/usr/bin/env python3

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import textwrap
import unittest


INSTALL_SCRIPT = Path(__file__).with_name("install.sh")
VERSION = "0.142.5"
DEFAULT_MANIFEST_URL = (
    "https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/"
    ".pipeline/release/vsp-release-manifest.json"
)


class InstallShTest(unittest.TestCase):
    def test_zero_argument_install_selects_matching_vsp_target(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, _checksum_path, _metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                vsp_source="preview-release-builder",
                manifest_url=DEFAULT_MANIFEST_URL,
            )
            archive_url = "https://releases.vsp.example/vsp-codex-macos.tar.gz"
            digest = hashlib.sha256(archive_path.read_bytes()).hexdigest()
            manifest_json = json.dumps(
                {
                    "schema": 1,
                    "minimum_updater": "0.134.0-vsp.6",
                    "releases": [
                        {
                            "version": "9.9.9-vsp.99",
                            "channel": "lab",
                            "target": "x86_64-unknown-linux-gnu",
                            "archive_url": "https://releases.vsp.example/decoy.tar.gz",
                            "sha256": "a" * 64,
                        },
                        {
                            "version": VERSION,
                            "channel": "lab",
                            "target": "aarch64-apple-darwin",
                            "archive_url": archive_url,
                            "sha256": digest,
                        },
                    ],
                }
            )

            result, requests = run_installer_in(
                root,
                None,
                archive_path=archive_path,
                force_macos=True,
                vsp_manifest_json=manifest_json,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(requests, [DEFAULT_MANIFEST_URL, archive_url])
            current = root / "codex-home" / "packages" / "standalone" / "current"
            self.assertEqual(
                subprocess.run(
                    [current / "bin" / "vsp-codex", "--version"],
                    capture_output=True,
                    check=True,
                    text=True,
                ).stdout,
                f"codex-cli {VERSION}\n",
            )

    def test_metadata_fetch_failure_is_not_reported_as_missing_assets(self) -> None:
        result, requests = run_installer(VERSION, metadata_failure=True)

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(
            requests,
            [
                "https://api.github.com/repos/openai/codex/releases/tags/"
                f"rust-v{VERSION}"
            ],
        )
        self.assertIn(
            f"Could not fetch GitHub release metadata for Codex {VERSION}",
            result.stderr,
        )
        self.assertNotIn("Could not find Codex package", result.stderr)

    def test_exact_release_fetches_metadata_once(self) -> None:
        result, requests = run_installer(VERSION)

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(
            requests,
            [
                "https://api.github.com/repos/openai/codex/releases/tags/"
                f"rust-v{VERSION}",
                "https://github.com/openai/codex/releases/download/"
                f"rust-v{VERSION}/codex-package_SHA256SUMS",
            ],
        )
        self.assertIn(f"Resolved version: {VERSION}", result.stdout)

    def test_vsp_alpha_release_is_accepted(self) -> None:
        release = "0.145.0-vsp.11.0-alpha.1"
        result, requests = run_installer(release)

        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("Invalid Codex release version", result.stderr)
        self.assertEqual(
            requests,
            [
                "https://api.github.com/repos/openai/codex/releases/tags/"
                f"rust-v{release}",
                "https://github.com/openai/codex/releases/download/"
                f"rust-v{release}/codex-package_SHA256SUMS",
            ],
        )
        self.assertIn(f"Resolved version: {release}", result.stdout)

    def test_latest_release_reuses_version_metadata(self) -> None:
        result, requests = run_installer("latest")

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(
            requests,
            [
                "https://api.github.com/repos/openai/codex/releases/latest",
                "https://github.com/openai/codex/releases/download/"
                f"rust-v{VERSION}/codex-package_SHA256SUMS",
            ],
        )
        self.assertIn(f"Resolved version: {VERSION}", result.stdout)

    def test_compact_metadata_is_independent_of_field_order(self) -> None:
        result, requests = run_installer(
            "latest", metadata_json=release_metadata(compact=True, reorder=True)
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(
            requests,
            [
                "https://api.github.com/repos/openai/codex/releases/latest",
                "https://github.com/openai/codex/releases/download/"
                f"rust-v{VERSION}/codex-package_SHA256SUMS",
            ],
        )
        self.assertIn(f"Resolved version: {VERSION}", result.stdout)

    def test_json_like_strings_and_nested_fields_do_not_define_assets(self) -> None:
        result, requests = run_installer(
            VERSION, metadata_json=legacy_release_metadata_with_decoys()
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(len(requests), 2)
        self.assertIn("/codex-npm-", requests[1])
        self.assertNotIn("codex-package_SHA256SUMS", requests[1])

    def test_macos_vsp_install_exposes_code_mode_host_beside_codex(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root, entrypoint="vsp-codex"
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            install_bin = root / "install-bin"
            current = root / "codex-home" / "packages" / "standalone" / "current"
            vsp_codex_path = install_bin / "vsp-codex"
            codex_path = install_bin / "codex"
            host_path = install_bin / "codex-code-mode-host"
            self.assertEqual(
                os.readlink(vsp_codex_path), str(current / "bin" / "vsp-codex")
            )
            self.assertFalse(codex_path.is_symlink())
            self.assertTrue(os.access(codex_path, os.X_OK))
            self.assertEqual(
                subprocess.run(
                    [codex_path, "--version"],
                    capture_output=True,
                    check=True,
                    text=True,
                ).stdout,
                f"codex-cli {VERSION}\n",
            )
            self.assertEqual(
                os.readlink(host_path),
                str(current / "bin" / "codex-code-mode-host"),
            )
            self.assertTrue(os.access(host_path, os.X_OK))
            self.assertFalse((current / "bin" / "codex").exists())
            self.assertTrue(os.access(current / "bin" / "vsp-codex", os.X_OK))
            with open(current / "codex-package.json", encoding="utf-8") as metadata:
                self.assertEqual(json.load(metadata)["entrypoint"], "bin/vsp-codex")

    def test_macos_install_refuses_to_rebrand_official_codex_package(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root, entrypoint="codex"
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn(
                "Refusing to install an official Codex package as VSP Codex",
                result.stderr,
            )

    def test_macos_vsp_install_rejects_official_metadata_entrypoint(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                metadata_entrypoint="codex",
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("VSP Codex package metadata is invalid", result.stderr)

    def test_macos_vsp_install_accepts_compact_package_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                compact_metadata=True,
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)

    def test_macos_vsp_direct_archive_validates_embedded_release_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, _checksum_path, _metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                compact_metadata=True,
            )

            result, requests = run_installer_in(
                root,
                VERSION,
                archive_path=archive_path,
                force_macos=True,
                direct_archive=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                requests,
                [
                    "https://releases.vsp.example/"
                    "codex-package-aarch64-apple-darwin.tar.gz"
                ],
            )

    def test_macos_vsp_direct_archive_accepts_release_builder_source_by_default(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, _checksum_path, _metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                vsp_source="preview-release-builder",
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                archive_path=archive_path,
                force_macos=True,
                direct_archive=True,
                release_source=None,
            )

            self.assertEqual(result.returncode, 0, result.stderr)

    def test_macos_vsp_install_runs_session_compat_for_existing_state(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root, entrypoint="vsp-codex"
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
                session_databases=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(
                "Synchronizing saved sessions with Codex Official", result.stdout
            )
            marker = root / "codex-home" / "session-compat-ran"
            self.assertIn("apply", marker.read_text(encoding="utf-8"))

    def test_macos_vsp_install_rejects_duplicate_release_metadata_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                duplicate_vsp_source=True,
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("duplicate JSON key: source", result.stderr)

    def test_macos_vsp_install_rejects_non_object_package_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                metadata_root_array=True,
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("package metadata root must be an object", result.stderr)

    def test_macos_vsp_install_rejects_invalid_package_json(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                invalid_metadata=True,
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Invalid VSP Codex package metadata", result.stderr)

    def test_macos_vsp_install_rejects_metadata_version_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                metadata_version="0.142.4",
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn(f"version must be {VERSION}", result.stderr)

    def test_macos_vsp_install_rejects_binary_version_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, checksum_path, metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                binary_version="0.142.4",
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata_json,
                archive_path=archive_path,
                checksum_path=checksum_path,
                force_macos=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn(
                f"binary version mismatch: expected {VERSION}, got 0.142.4",
                result.stderr,
            )

    def test_macos_vsp_direct_archive_rejects_source_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path, _checksum_path, _metadata_json = create_package_release(
                root,
                entrypoint="vsp-codex",
                vsp_source="unexpected-source",
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                archive_path=archive_path,
                force_macos=True,
                direct_archive=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("vspRelease.source must be installer-test", result.stderr)


def run_installer(
    release: str,
    *,
    metadata_failure: bool = False,
    metadata_json: str | None = None,
) -> tuple[subprocess.CompletedProcess[str], list[str]]:
    with tempfile.TemporaryDirectory() as temp_dir:
        return run_installer_in(
            Path(temp_dir),
            release,
            metadata_failure=metadata_failure,
            metadata_json=metadata_json,
        )


def run_installer_in(
    root: Path,
    release: str | None,
    *,
    metadata_failure: bool = False,
    metadata_json: str | None = None,
    archive_path: Path | None = None,
    checksum_path: Path | None = None,
    force_macos: bool = False,
    direct_archive: bool = False,
    release_source: str | None = "installer-test",
    session_databases: bool = False,
    vsp_manifest_json: str | None = None,
) -> tuple[subprocess.CompletedProcess[str], list[str]]:
    bin_dir = root / "bin"
    bin_dir.mkdir()
    request_log = root / "requests.log"
    fake_curl = bin_dir / "curl"
    fake_curl.write_text(
        textwrap.dedent(
            """\
            #!/bin/sh
            url=""
            output=""
            previous=""
            for arg in "$@"; do
              case "$arg" in
                https://*) url="$arg" ;;
              esac
              if [ "$previous" = "-o" ]; then
                output="$arg"
              fi
              previous="$arg"
            done
            printf '%s\n' "$url" >>"$CODEX_TEST_REQUEST_LOG"

            case "$url" in
              https://api.github.com/*)
                if [ "$CODEX_TEST_METADATA_FAILURE" = "1" ]; then
                  echo "curl: (22) The requested URL returned error: 403" >&2
                  exit 22
                fi
                printf '%s\n' "$CODEX_TEST_METADATA_JSON"
                ;;
              */vsp-release-manifest.json)
                printf '%s\n' "$CODEX_TEST_VSP_MANIFEST_JSON"
                ;;
              */codex-package_SHA256SUMS)
                if [ -n "$CODEX_TEST_CHECKSUM_PATH" ]; then
                  cp "$CODEX_TEST_CHECKSUM_PATH" "$output"
                else
                  exit 22
                fi
                ;;
              */codex-package-*.tar.gz)
                if [ -n "$CODEX_TEST_ARCHIVE_PATH" ]; then
                  cp "$CODEX_TEST_ARCHIVE_PATH" "$output"
                else
                  exit 22
                fi
                ;;
              *.tar.gz)
                if [ -n "$CODEX_TEST_ARCHIVE_PATH" ]; then
                  cp "$CODEX_TEST_ARCHIVE_PATH" "$output"
                else
                  exit 22
                fi
                ;;
              *)
                exit 22
                ;;
            esac
            """
        ),
        encoding="utf-8",
    )
    fake_curl.chmod(0o755)
    if force_macos:
        fake_uname = bin_dir / "uname"
        fake_uname.write_text(
            "#!/bin/sh\n"
            'case "$1" in\n'
            "  -s) printf 'Darwin\\n' ;;\n"
            "  -m) printf 'arm64\\n' ;;\n"
            "esac\n",
            encoding="utf-8",
        )
        fake_uname.chmod(0o755)

    home = root / "home"
    home.mkdir()
    if session_databases:
        (root / "codex-home").mkdir()
        (root / "codex-home" / "state_5.sqlite").touch()
        (home / ".codex").mkdir()
        (home / ".codex" / "state_5.sqlite").touch()
    env = os.environ.copy()
    env.update(
        {
            "CODEX_HOME": str(root / "codex-home"),
            "CODEX_INSTALL_DIR": str(root / "install-bin"),
            "CODEX_NON_INTERACTIVE": "1",
            "CODEX_TEST_ARCHIVE_PATH": str(archive_path or ""),
            "CODEX_TEST_CHECKSUM_PATH": str(checksum_path or ""),
            "CODEX_TEST_METADATA_FAILURE": "1" if metadata_failure else "0",
            "CODEX_TEST_METADATA_JSON": (
                metadata_json if metadata_json is not None else release_metadata()
            ),
            "CODEX_TEST_REQUEST_LOG": str(request_log),
            "CODEX_TEST_VSP_MANIFEST_JSON": vsp_manifest_json or "{}",
            "HOME": str(home),
            "PATH": f"{bin_dir}:/usr/bin:/bin",
            "SHELL": "/bin/sh",
        }
    )
    if release is not None:
        env["CODEX_RELEASE"] = release
    else:
        env.pop("CODEX_RELEASE", None)
    if direct_archive:
        if archive_path is None:
            raise ValueError("direct_archive requires archive_path")
        env.update(
            {
                "VSP_CODEX_ARCHIVE_URL": (
                    "https://releases.vsp.example/"
                    "codex-package-aarch64-apple-darwin.tar.gz"
                ),
                "VSP_CODEX_SHA256": hashlib.sha256(
                    archive_path.read_bytes()
                ).hexdigest(),
                "VSP_CODEX_TARGET": "aarch64-apple-darwin",
                "VSP_CODEX_VERSION": VERSION,
                "VSP_CODEX_RELEASE_MANIFEST_URL": (
                    "https://releases.vsp.example/manifest.json"
                ),
                "VSP_CODEX_RELEASE_CHANNEL": "lab",
            }
        )
        if release_source is not None:
            env["VSP_CODEX_RELEASE_SOURCE"] = release_source
    result = subprocess.run(
        ["/bin/sh", str(INSTALL_SCRIPT)],
        capture_output=True,
        check=False,
        env=env,
        text=True,
    )
    requests = (
        request_log.read_text(encoding="utf-8").splitlines()
        if request_log.exists()
        else []
    )
    return result, requests


def create_package_release(
    root: Path,
    *,
    entrypoint: str,
    metadata_entrypoint: str | None = None,
    compact_metadata: bool = False,
    duplicate_vsp_source: bool = False,
    metadata_root_array: bool = False,
    invalid_metadata: bool = False,
    vsp_source: str = "installer-test",
    metadata_version: str = VERSION,
    binary_version: str = VERSION,
    manifest_url: str = "https://releases.vsp.example/manifest.json",
) -> tuple[Path, Path, str]:
    package_dir = root / "package"
    (package_dir / "bin").mkdir(parents=True)
    (package_dir / "codex-path").mkdir()
    (package_dir / "codex-resources").mkdir()
    package_metadata = {
        "version": metadata_version,
        "entrypoint": f"bin/{metadata_entrypoint or entrypoint}",
        "vspRelease": {
            "manifestUrl": manifest_url,
            "channel": "lab",
            "source": vsp_source,
            "target": "aarch64-apple-darwin",
        },
    }
    if metadata_root_array:
        package_metadata_text = json.dumps([package_metadata])
    else:
        package_metadata_text = json.dumps(
            package_metadata,
            indent=None if compact_metadata else 2,
            separators=(",", ":") if compact_metadata else None,
        )
    if duplicate_vsp_source:
        package_metadata_text = package_metadata_text.replace(
            f'"source": "{vsp_source}"',
            f'"source": "{vsp_source}", "source": "duplicate"',
            1,
        )
    if invalid_metadata:
        package_metadata_text += " trailing"
    (package_dir / "codex-package.json").write_text(
        package_metadata_text + "\n",
        encoding="utf-8",
    )
    write_executable(
        package_dir / "bin" / entrypoint,
        f"#!/bin/sh\nprintf 'codex-cli {binary_version}\\n'\n",
    )
    write_executable(
        package_dir / "bin" / "codex-code-mode-host",
        "#!/bin/sh\nexit 0\n",
    )
    write_executable(package_dir / "codex-path" / "rg", "#!/bin/sh\nexit 0\n")
    write_executable(
        package_dir / "codex-resources" / "vsp-session-compat.py",
        """#!/usr/bin/env python3
import sys
from pathlib import Path

args = sys.argv[1:]
vsp_home = Path(args[args.index("--vsp-home") + 1])
(vsp_home / "session-compat-ran").write_text(" ".join(args), encoding="utf-8")
""",
    )

    asset = "codex-package-aarch64-apple-darwin.tar.gz"
    archive_path = root / asset
    with tarfile.open(archive_path, "w:gz") as archive:
        for path in package_dir.iterdir():
            archive.add(path, arcname=path.name)

    archive_digest = hashlib.sha256(archive_path.read_bytes()).hexdigest()
    checksum_path = root / "codex-package_SHA256SUMS"
    checksum_path.write_text(f"{archive_digest}  {asset}\n", encoding="utf-8")
    checksum_digest = hashlib.sha256(checksum_path.read_bytes()).hexdigest()
    metadata_json = json.dumps(
        {
            "assets": [
                {"name": asset, "digest": f"sha256:{archive_digest}"},
                {
                    "name": "codex-package_SHA256SUMS",
                    "digest": f"sha256:{checksum_digest}",
                },
            ],
            "tag_name": f"rust-v{VERSION}",
        },
        indent=2,
    )
    return archive_path, checksum_path, metadata_json


def write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


def release_metadata(*, compact: bool = False, reorder: bool = False) -> str:
    assets = [
        asset_metadata(
            f"codex-package-{target}.tar.gz",
            f"sha256:{'a' * 64}",
            reorder=reorder,
        )
        for target in (
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "aarch64-unknown-linux-musl",
            "x86_64-unknown-linux-musl",
        )
    ]
    assets.append(
        asset_metadata(
            "codex-package_SHA256SUMS",
            f"sha256:{'b' * 64}",
            reorder=reorder,
        )
    )
    separators = (",", ":") if compact else None
    return json.dumps(
        {"assets": assets, "body": "braces: { } [ ]", "tag_name": f"rust-v{VERSION}"},
        indent=None if compact else 2,
        separators=separators,
    )


def asset_metadata(name: str, digest: str, *, reorder: bool) -> dict[str, str]:
    if reorder:
        return {"digest": digest, "name": name}
    return {"name": name, "digest": digest}


def legacy_release_metadata_with_decoys() -> str:
    fake_digest = f"sha256:{'0' * 64}"
    assets = [
        {
            "metadata": {
                "name": "codex-package-x86_64-unknown-linux-musl.tar.gz",
                "digest": fake_digest,
            },
            "digest": f"sha256:{'c' * 64}",
            "name": f"codex-npm-{target}-{VERSION}.tgz",
        }
        for target in ("darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64")
    ]
    return json.dumps(
        {
            "body": (
                f'fake: {{"name":"codex-package_SHA256SUMS","digest":"{fake_digest}"}}'
            ),
            "assets": assets,
            "tag_name": f"rust-v{VERSION}",
        },
        separators=(",", ":"),
    )


if __name__ == "__main__":
    unittest.main()
