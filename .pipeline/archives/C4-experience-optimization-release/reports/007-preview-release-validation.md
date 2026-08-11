# C4/M7 Preview Release Validation

Updated: 2026-05-28T16:55:00+08:00

## Summary

M7 produced a Linux x64 preview package and collated the focused release evidence for the C4 preview. This is a GNU debug/dev preview artifact assembled from the existing local `codex-rs/target/debug/vsp-codex` binary. It is usable for preview smoke testing on compatible Linux x64/glibc environments, but it is not presented as the official MUSL release artifact.

## Preview Artifact

Artifact:

- `.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`

Checksum:

- `.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256`
- SHA-256: `75921ba46926ffac01cc964bb29b792e1dadb052880d5dae5df59de953d771e7`

Package metadata:

- `version`: `0.134.0-vsp.4`
- `target`: `x86_64-unknown-linux-gnu`
- `variant`: `codex`
- `entrypoint`: `bin/vsp-codex`
- archive size: `264808693` bytes

Included package files verified:

- `codex-package.json`
- `bin/vsp-codex`
- `codex-path/rg`
- `codex-resources/bwrap`
- `codex-resources/zsh/bin/zsh`

Binary/resource sizes observed:

- `bin/vsp-codex`: 1.2G debug binary
- `codex-path/rg`: 27M
- `codex-resources/bwrap`: 72K
- `codex-resources/zsh/bin/zsh`: 880K

## Artifact Validation

Build command:

```sh
python3 scripts/build_codex_package.py \
  --target x86_64-unknown-linux-gnu \
  --variant codex \
  --package-dir .pipeline/release/C4-preview/package \
  --archive-output .pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz \
  --force \
  --entrypoint-bin codex-rs/target/debug/vsp-codex \
  --bwrap-bin /usr/bin/bwrap \
  --rg-bin /home/heyx/.cargo/bin/rg
```

Result: passed.

Smoke commands:

```sh
.pipeline/release/C4-preview/package/bin/vsp-codex --version
```

Result: `vsp-codex 0.134.0-vsp.4`.

```sh
rm -rf /tmp/vsp-codex-preview-smoke
mkdir -p /tmp/vsp-codex-preview-smoke
tar -xzf .pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz \
  -C /tmp/vsp-codex-preview-smoke
/tmp/vsp-codex-preview-smoke/bin/vsp-codex --version
```

Result: `vsp-codex 0.134.0-vsp.4`.

```sh
sha256sum .pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz \
  > .pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256
```

Result: checksum file written.

## Curl Install Path

README now documents the preview install command against the project GitLab raw path:

```sh
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- \
  --archive-url https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz \
  --sha256 75921ba46926ffac01cc964bb29b792e1dadb052880d5dae5df59de953d771e7 \
  --target x86_64-unknown-linux-gnu \
  --version 0.134.0-vsp.4-preview \
  --vscode \
  --no-launch \
  --yes
```

Unauthenticated `curl -I -L` against the raw URL currently redirects to `/users/sign_in`, so direct copy/paste installation requires one of:

- GitLab raw download access enabled for the intended audience.
- Publishing the archive as a GitLab Release/Package Registry asset with direct download access.
- Using a GitLab token-bearing download command.

## Focused Test Matrix

Source: `.pipeline/reviews/C4/M7/test.md`.

- `cd codex-rs && just fmt`: passed.
- `cd codex-rs && just test -p codex-cli vscode`: passed, 8 passed / 225 skipped.
- `cd codex-rs && just test -p codex-app-server initialize`: passed, 14 passed / 777 skipped.
- `cd codex-rs && just test -p codex-app-server thread_read`: passed, 18 passed / 773 skipped.
- `cd codex-rs && just test -p codex-tui startup`: passed, 45 passed / 2649 skipped.
- `cd codex-rs && just test -p codex-tui keymap`: passed, 108 passed / 2586 skipped.
- `cd codex-rs && just test -p codex-tui markdown_render`: passed, 126 passed / 2568 skipped.
- `cargo insta pending-snapshots --manifest-path codex-rs/tui/Cargo.toml`: passed, no pending snapshots.
- `git diff --check`: passed after the M7 test retry hygiene fix.
- `just test -p codex-core load_config_ignores_empty_model_provider_override`: passed for vsp.4 hotfix.
- `just test -p codex-core load_config_uses_default_model_provider_when_configured_provider_is_empty`: passed for vsp.4 hotfix.
- `just test -p codex-app-server merge_persisted_resume_metadata_ignores_empty_model_provider`: passed for vsp.4 hotfix.

Full workspace `just test` was not run because repository rules require asking the user before running the complete suite.

## VSCode Install/Status/Rollback Evidence

Source: `.pipeline/reviews/C4/M7/test.md`.

Manual isolated smoke with temp `HOME`, `XDG_CONFIG_HOME`, `CODEX_HOME`, and fake `code` binary passed:

- `vsp-codex vscode install --yes` wrote `chatgpt.cliExecutable` and `chatgpt.localeOverride`.
- `vsp-codex vscode status` reported configured user scope and executable path.
- `vsp-codex vscode rollback --yes` restored original settings.
- Final settings were restored to `{"editor.fontSize": 15, "workbench.colorTheme": "Default Dark Modern"}` with one backup retained.

## Performance Evidence

Artifact:

- `.pipeline/evidence/performance-startup.json`

Latest smoke result:

- `cold_tui_start`: `skipped`; only terminal control output was observed before timeout, so no first-visible-screen duration was claimed.
- `resume_list_first_page`: `passed`, 5 ms after initialize.
- `open_historical_session`: `skipped`; no real historical thread id/home supplied.
- `vscode_app_server_initialize`: `passed`, 304 ms.

## Known Limitations

- This preview artifact is Linux x64 GNU/glibc debug/dev output, not a MUSL release build. It was assembled from the existing local debug binary to satisfy the preview hard gate without claiming production release parity.
- The archive is large because `bin/vsp-codex` is an unstripped debug binary.
- The README curl command uses the GitLab raw path, but direct unauthenticated curl currently depends on GitLab access policy; test showed redirect to `/users/sign_in`.
- VSCode settings handling is strict JSON; JSONC comments, trailing commas, and original formatting are not preserved.
- Local extension presence is simulated in tests. Real end-user validation still needs a machine with the official extension path or explicit extension install instructions.
- `cold_tui_start` does not have usable first-visible-screen timing in this environment.
- `open_historical_session` requires `CODEX_PERF_THREAD_ID` and `CODEX_PERF_EXISTING_CODEX_HOME` pointed at real history to produce release-grade timing.
- Complete workspace `just test` was intentionally not run under the repository rule requiring user approval.

## Release Decision

Preview release evidence is sufficient for an internal Linux x64 preview package, with the GNU debug target and the skipped performance scenarios called out explicitly. It is not sufficient to label the artifact as an official MUSL production release.
