# v0.140.0-vsp.8.1 Release Notes

Status: published internal preview. The tag, GitLab Release, release asset, SHA256 asset, and stable manifest entry are available for `v0.140.0-vsp.8.1`.

## Upstream Baseline

- Upstream Codex Rust baseline: `rust-v0.143.0-alpha.25`.
- VSP version semantics remain in the `vsp.8` series: this is a patch-level VSP preview release on top of the existing `0.140.0-vsp.8` line, not a reset to a new VSP sequence.
- alpha.25 differences that are not VSP-equivalent or are too broad for this preview release remain preserve/defer items; this release should not be described as a full upstream alpha.25 merge.

## Highlights

- SQLite log write convergence: noisy persistent log targets such as `log`, `codex_otel.log_only`, and `codex_otel.trace_safe` are filtered, with the VSP in-layer `target=log` guard retained to reduce SQLite write amplification.
- Plugin/Remote APP session visibility startup path fix: VSCode Remote bundled `codex` shim drift is diagnosed, and Remote APP-compatible `codex` startup can route back to VSP Codex so VSP sessions remain visible from the expected state store.
- VSCode bundled codex shim and diagnostics: `vsp-codex vscode install/status/rollback` continue to maintain the official ChatGPT extension bundled CLI shim and report drift instead of silently falling back to an upstream Codex binary.
- Installer and updater compatibility: VSP keeps maintaining both `vsp-codex` and `codex` compatible entrypoints. Explicit VSP manifest updates remain available, while ordinary upstream update paths stay disabled by the VSP gate.

## Install Command

The release asset URL below is the GitLab Release downloads URL. The SHA256 matches the package generated and uploaded during M12.

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- \
  --archive-url https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.140.0-vsp.8.1/downloads/vsp-codex-preview-linux-x64-gnu.tar.gz \
  --sha256 0ff6e0d34daf6265e23cb99d21b69d9acbbcce4df2067c168f72f0e7a82e567e \
  --target x86_64-unknown-linux-gnu \
  --version 0.140.0-vsp.8.1-preview \
  --manifest-url https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json \
  --vscode \
  --no-launch \
  --yes
```

## Release Status

- Version metadata published for `0.140.0-vsp.8.1`.
- User-visible docs and stable manifest point to GitLab tag `v0.140.0-vsp.8.1`.
- Release asset: `vsp-codex-preview-linux-x64-gnu.tar.gz`.
- SHA256: `0ff6e0d34daf6265e23cb99d21b69d9acbbcce4df2067c168f72f0e7a82e567e`.
- Release URL: `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.140.0-vsp.8.1`.
