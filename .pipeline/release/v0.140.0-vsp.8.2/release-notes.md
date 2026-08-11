# v0.140.0-vsp.8.2 Release Notes

Status: published internal preview. The tag, GitLab Release, release asset, SHA256 asset, and stable manifest entry are available for `v0.140.0-vsp.8.2`.

## Upstream Baseline

- Upstream Codex Rust baseline remains `rust-v0.143.0-alpha.25`.
- VSP version semantics remain in the `vsp.8` series: this is a patch-level VSP preview release on top of the existing `0.140.0-vsp.8` line, not a reset to a new VSP sequence.
- alpha.25 differences that are not VSP-equivalent or are too broad for this preview release remain preserve/defer items; this release should not be described as a full upstream alpha.25 merge.

## Highlights

- C20 consultation-first action boundary: `discussion`, `background`, `idea`, `complaint`, `question`, and `solution-discussion` inputs are treated as non-editing signals.
- Non-editing signals first receive a Mini-contract in this order: `我的理解` -> `问题原因` -> `推荐方案`.
- Clear imperative requests with a concrete target may still proceed directly.
- After a plan, Mini-contract, or recommendation has been shown, replies such as `可以`, `确认`, `OK`, `go ahead`, or `apply it` authorize only the scope already shown to the user.
- The boundary is synchronized across root Workflow/OpenCode guidance, OpenCode agents, plan command adapters, bundled model catalog instructions, and user-facing docs.
- Plugin/Remote distribution bindings are preserved: the package metadata keeps the VSP manifest URL, `lab` channel, Linux GNU target, `bin/vsp-codex` entrypoint, and compatibility `codex` wrapper path.
- VSCode repair/self-heal is included: `vsp-codex vscode repair --yes` can repair remote bundled Codex shims without changing VSCode settings backups, and the wrapper quietly self-heals once on launch.
- App-server remote socket remap is included: official `~/.codex/app-server-control` socket paths are remapped to `~/.vsp-codex/app-server-control`.
- Thread API provider fallback is included: blank or unknown persisted `model_provider` values are normalized to the configured provider before clients receive thread metadata.
- Installer compatibility is included: the compatibility `codex` entrypoint is a wrapper that pins `CODEX_HOME` to `~/.vsp-codex` by default.

## Install Command

The release asset URL below is the GitLab Release downloads URL. The SHA256 matches the package generated and uploaded for this release.

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- \
  --archive-url https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.140.0-vsp.8.2/downloads/vsp-codex-preview-linux-x64-gnu.tar.gz \
  --sha256 427a101bfecfa123c6795c12d6a2ca4b5a26952b6f6b61ba9a654106e20cb65b \
  --target x86_64-unknown-linux-gnu \
  --version 0.140.0-vsp.8.2-preview \
  --manifest-url https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json \
  --vscode \
  --no-launch \
  --yes
```

## Release Status

- Version metadata published for `0.140.0-vsp.8.2`.
- User-visible docs and stable manifest point to GitLab tag `v0.140.0-vsp.8.2`.
- Release asset: `vsp-codex-preview-linux-x64-gnu.tar.gz`.
- SHA256: `427a101bfecfa123c6795c12d6a2ca4b5a26952b6f6b61ba9a654106e20cb65b`.
- Release URL: `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.140.0-vsp.8.2`.

## Validation

- JSON and TypeScript/JavaScript syntax checks for OpenCode and Hypo-Workflow adapter surfaces.
- `codex-models-manager` bundled model regression for the consultation-first instruction paths.
- Release build and package generation.
- Package SHA256 verification.
- Isolated installer smoke confirming both `vsp-codex` and compatibility `codex` entrypoints.
- Package metadata verification confirming VSP update manifest URL, channel, target, and entrypoint.
- VSCode CLI binding regression confirming plugin binding status remains configured after distribution.
- App-server regression confirming unknown historical providers fall back to the configured provider for client-facing thread metadata.

## Local Redistribution Result

- Local `~/.vsp-codex` was updated from `0.140.0-vsp.8.1` to `0.140.0-vsp.8.2-preview` using the generated archive.
- `/home/heyx/.local/bin/vsp-codex --version` returns `vsp-codex 0.140.0-vsp.8.2`.
- `/home/heyx/.local/bin/codex --version` returns `vsp-codex 0.140.0-vsp.8.2`.
- VSCode status remains configured with `cliExecutable=/home/heyx/.vsp-codex/vscode/vsp-codex`; the previous wrapper version-drift warning is resolved.
- Existing remote VSCode ChatGPT bundled `codex` executables under `.vscode-server/extensions/openai.chatgpt-*` were repointed to the VSP wrapper.
- Remaining VSCode warning is the expected Remote Machine settings scope notice: VSCode ignores application-scoped settings written to remote Machine settings unless the local VSCode UI User settings also contain the same values.
