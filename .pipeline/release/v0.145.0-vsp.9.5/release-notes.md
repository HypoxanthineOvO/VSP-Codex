# v0.145.0-vsp.9.5

**Recovery release**: make the installed version, package metadata, release manifest, and actual binary agree.

## What changed

- Zero-argument `install.sh` now resolves the latest VSP package for the current platform and `lab` channel. Explicit `CODEX_RELEASE` or `--release` continues to select an official Codex release.
- The installer validates the manifest selection, archive SHA-256, package metadata version, actual `vsp-codex --version`, existing release directories, and both visible command entries before reporting success.
- The release builder rejects workspace/npm version drift, a mismatched prebuilt binary, or a package whose metadata, target, entrypoint, and binary do not match the release version.
- The Rust updater rejects mislabeled or stale installed release directories and preserves the original `preview-release-builder` provenance after an update.
- README and installation docs now use the parameter-free installation command and require checking both `vsp-codex --version` and `codex --version`.

## Install or recover

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh
```

After installation, both commands must report the same version:

```shell
vsp-codex --version
codex --version
```

Expected output for both entries is `vsp-codex 0.145.0-vsp.9.5`. The current release manifest provides a Linux x64 GNU/glibc package.

## Validation

- Release binary and extracted archive binary both report `0.145.0-vsp.9.5` and have identical SHA-256 digests.
- Archive SHA-256: `ab1ce3615107c4aff70c1120bd8a2d19b26eea30ebb774623bb86313d07444a8`.
- A clean isolated installation exposes matching `vsp-codex` and `codex` commands.
- An intentionally mislabeled 9.5 directory containing the previously published 9.3 binary is detected as incomplete, downloaded again, and repaired to 9.5.
- A 9.5 installation checked against the 9.5 manifest reports that no newer VSP release matches the install.
