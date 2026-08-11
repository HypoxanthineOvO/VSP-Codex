# C8/M4 Implement Worker Report

## Files Changed

- `codex-rs/rollout/src/lib.rs`
- `codex-rs/rollout/src/official_codex_discovery.rs`
- `.pipeline/reviews/C8/M4/implement.md`

## Implementation Summary

Implemented `codex_rollout::official_codex_discovery` with the public API required by the red tests:

- `export_sessions_for_official_codex(vsp_codex_home, official_codex_home)`
- `classify_field_for_official_discovery(field_path)`
- `OfficialCodexExport`
- `OfficialCodexSessionExport`
- `OfficialDiscoveryFieldClass`

The exporter discovers VSP rollout JSONL files under `sessions/**`, writes real sanitized JSONL artifacts under the target official Codex home at `official_codex_discovery/sessions/`, and writes a manifest at `official_codex_discovery/manifest.json`. It does not symlink, raw-share, or expose the source VSP session tree.

Safe resume/listing metadata is preserved for official discovery:

- session id
- created timestamp
- cwd
- title/name from `session_index.jsonl` or safe rollout metadata
- source
- sanitized artifact path

## Privacy Strategy

The sanitizer parses each rollout JSONL line as JSON, recursively removes fields classified as private or secret, and keeps only a narrow allowlist for `session_meta` payload data. Provider/auth/config/env/workflow/rollout trace fields are stripped before writing official-visible artifacts.

Classification is field-path based:

- safe: session id, timestamp, cwd, source, session index title/name, sanitized export path
- private: provider/config/model provider details, workflow `.pipeline` state, rollout trace metadata, private implementation markers
- secret: API keys, auth headers, refresh/access tokens, authorization values, secret-looking environment keys

String values that look like raw secrets or private workflow/provider markers are cleared even when they appear in otherwise safe event payloads.

## Validation

Focused tests:

```sh
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-rollout --test official_codex_discovery
```

Result: passed, 4 tests.

Formatting:

```sh
cargo fmt --check --package codex-rollout
```

Result: passed. The command prints the repository's existing stable-toolchain warning for the unstable rustfmt option `imports_granularity = Item`.

Whitespace:

```sh
git diff --check
```

Result: passed.

## Fix-Round Implementation

Files changed:

- `codex-rs/rollout/src/official_codex_discovery.rs`
- `.pipeline/reviews/C8/M4/implement.md`

What changed:

- Sanitized `session_index.jsonl.thread_name` before assigning it to `OfficialCodexSessionExport.title` or writing the manifest. Titles containing suspicious/private markers such as `.pipeline`, `/hw:`, provider markers, `private`, or `sk-*` are now cleared instead of exported.
- Applied the same title sanitizer to rollout `session_meta.payload.title` / `name` when used as fallback title metadata.
- Made sanitized artifact paths collision-free for source rollout files with the same basename by deriving the destination directory from a sanitized relative source path plus a stable short hash. Artifacts are still real JSONL files under official CODEX_HOME and preserve the source rollout basename.

Validation:

```sh
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-rollout --test official_codex_discovery
```

Result: passed, 8 tests.

```sh
cargo fmt --check --package codex-rollout
```

Result: passed. The command emitted the repository's existing stable-toolchain warning for unstable rustfmt option `imports_granularity = Item`.

```sh
git diff --check
```

Result: passed.
