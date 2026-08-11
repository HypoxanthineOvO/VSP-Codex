# C4/M7 Implement Worker Report

worker: implement
milestone: M7 Preview Release Validation
scope: release artifact assembly, checksum, artifact smoke validation, and release report collation only.

## Files Changed

- `.pipeline/release/C4-preview/package/`
- `.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`
- `.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256`
- `.pipeline/reports/007-preview-release-validation.md`
- `.pipeline/reviews/C4/M7/implement.md`

No tests, snapshots, or production runtime code were edited by this worker.

## Artifact

Path:

- `.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`

Checksum:

- `0cb38d6fc23de3aa57e125af5862f67b5400dc91009c5168e977e150b73be557`

Checksum file:

- `.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256`

Artifact type:

- target: `x86_64-unknown-linux-gnu`
- profile/source: existing local debug binary at `codex-rs/target/debug/vsp-codex`
- limitation: GNU debug/dev preview, not official MUSL release

## Commands Run

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

```sh
tar -tzf .pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz
```

Result: passed. Verified archive includes `codex-package.json`, `bin/vsp-codex`, `codex-path/rg`, `codex-resources/bwrap`, and `codex-resources/zsh/bin/zsh`.

```sh
python3 -m json.tool .pipeline/release/C4-preview/package/codex-package.json
```

Result: passed. Metadata reports `version` `0.134.0-vsp.2`, `target` `x86_64-unknown-linux-gnu`, `variant` `codex`, and `entrypoint` `bin/vsp-codex`.

```sh
find .pipeline/release/C4-preview/package -maxdepth 3 -type f -printf '%M %s %p\n' | sort
```

Result: passed. Verified executable permissions for `bin/vsp-codex`, `codex-path/rg`, and `codex-resources/bwrap`.

```sh
timeout 20 .pipeline/release/C4-preview/package/bin/vsp-codex --version
```

Result: passed, `vsp-codex 0.134.0-vsp.2`.

```sh
sha256sum .pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz \
  > .pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256
```

Result: passed.

```sh
rm -rf /tmp/vsp-codex-preview-smoke
mkdir -p /tmp/vsp-codex-preview-smoke
tar -xzf .pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz \
  -C /tmp/vsp-codex-preview-smoke
/tmp/vsp-codex-preview-smoke/bin/vsp-codex --version
```

Result: passed, `vsp-codex 0.134.0-vsp.2`.

```sh
python3 -m json.tool .pipeline/evidence/performance-startup.json
```

Result: passed.

## Evidence Collated

Focused M7 validation came from `.pipeline/reviews/C4/M7/test.md`:

- CLI VSCode tests passed.
- app-server initialize and thread read tests passed.
- TUI startup, keymap, and Markdown render tests passed.
- VSCode install/status/rollback isolated smoke passed.
- TUI snapshots had no pending changes.
- `git diff --check` passed after test retry hygiene cleanup.

Performance evidence came from `.pipeline/evidence/performance-startup.json`:

- `resume_list_first_page`: passed.
- `vscode_app_server_initialize`: passed.
- `cold_tui_start`: skipped honestly because only terminal control output was observed before timeout.
- `open_historical_session`: skipped honestly because no real historical thread id/home was supplied.

## Known Limitations

- The artifact is a Linux x64 GNU debug/dev preview, not an official MUSL release package.
- The archive is large due to the debug `vsp-codex` binary.
- VSCode settings parsing is strict JSON.
- Extension presence is simulated in local tests.
- Full workspace `just test` was not run because repository rules require user approval.
