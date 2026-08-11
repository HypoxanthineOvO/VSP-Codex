# C8/M4 Audit Recheck Report

Verdict: pass

Recheck timestamp: 2026-05-30T17:59:00+08:00

Audit scope:

- `.pipeline/reviews/C8/M4/audit.md`
- `.pipeline/reviews/C8/M4/test.md`
- `.pipeline/reviews/C8/M4/implement.md`
- `codex-rs/rollout/tests/official_codex_discovery.rs`
- `codex-rs/rollout/src/official_codex_discovery.rs`

Worker boundary:

- Read-only recheck of production code and tests.
- This report is the only file written by this recheck worker.
- Protected `.pipeline/state.yaml`, `.pipeline/cycle.yaml`, and `.pipeline/rules.yaml` were not edited.

## Findings

No blocking findings remain.

### Resolved: `session_index.jsonl.thread_name` leakage

References:

- `codex-rs/rollout/src/official_codex_discovery.rs:371`
- `codex-rs/rollout/src/official_codex_discovery.rs:384`
- `codex-rs/rollout/src/official_codex_discovery.rs:476`
- `codex-rs/rollout/tests/official_codex_discovery.rs:126`

`session_index.jsonl.thread_name` is still read as the preferred title source, but it now passes through `sanitize_export_title` before becoming `OfficialCodexSessionExport.title` or manifest JSON. The sanitizer rejects suspicious/private markers via `looks_like_secret_value`, including `.pipeline`, `/hw:`, provider markers, private markers, and `sk-*`-style secrets.

Regression coverage is present in `session_index_thread_name_is_sanitized_before_manifest_title`, which verifies both the returned export title and manifest JSON do not contain the unsafe indexed title markers.

### Resolved: duplicate basename artifact collision

References:

- `codex-rs/rollout/src/official_codex_discovery.rs:281`
- `codex-rs/rollout/src/official_codex_discovery.rs:289`
- `codex-rs/rollout/src/official_codex_discovery.rs:513`
- `codex-rs/rollout/tests/official_codex_discovery.rs:228`

Sanitized artifact paths are no longer derived from basename alone. `artifact_path_for_candidate` prefixes the destination with a stable hash of the candidate relative path and sanitized parent path components, while preserving the source rollout basename.

Regression coverage is present in `duplicate_source_basenames_do_not_overwrite_sanitized_artifacts`, which creates two source rollouts with the same basename, asserts distinct sanitized artifact paths, and verifies both session ids remain present in exported artifacts.

### Resolved: sanitized artifact parses through normal rollout reader

References:

- `codex-rs/rollout/src/official_codex_discovery.rs:301`
- `codex-rs/rollout/src/official_codex_discovery.rs:338`
- `codex-rs/rollout/tests/official_codex_discovery.rs:199`

The sanitized artifact remains readable by the existing rollout reader. Regression coverage is present in `sanitized_artifact_is_readable_by_rollout_thread_reader`, which calls `codex_rollout::read_thread_item_from_rollout` on the exported sanitized artifact and verifies thread id, cwd, preview, and source.

### Resolved: no output symlink/raw-share of VSP home

References:

- `codex-rs/rollout/src/official_codex_discovery.rs:124`
- `codex-rs/rollout/src/official_codex_discovery.rs:146`
- `codex-rs/rollout/src/official_codex_discovery.rs:301`
- `codex-rs/rollout/tests/official_codex_discovery.rs:96`

The exporter writes real sanitized files under `official_codex_home/official_codex_discovery/sessions/` and writes a manifest under `official_codex_home/official_codex_discovery/manifest.json`. It does not symlink or raw-share the VSP home.

Regression coverage is present in `export_for_official_codex_does_not_symlink_or_raw_share_vsp_home`, which verifies no symlink in the official export tree points into the VSP home, sanitized artifacts are real files, and the official `sessions` directory is not the raw VSP sessions directory.

### Residual risk: source symlink traversal relies on current filesystem behavior

References:

- `codex-rs/rollout/src/official_codex_discovery.rs:225`
- `codex-rs/rollout/src/official_codex_discovery.rs:236`
- `codex-rs/rollout/tests/official_codex_discovery.rs:159`

The collector still uses `std::fs::DirEntry::metadata()` while recursively walking VSP `sessions/**`. On this Linux validation environment, the regression test `symlinked_sessions_directory_outside_root_is_not_traversed` passes and no current testable exploit was reproduced. Based on current behavior, a symlinked directory placed under `sessions/` was not exported.

Residual risk remains because the implementation does not explicitly use `symlink_metadata()` or canonical root containment checks in the collector. If this code must defend against untrusted session directory contents across platforms/filesystems, a future hardening patch should make symlink skipping/root containment explicit. This is not marked `needs_changes` for M4 because the requested regression test passes and no current exploit was found.

### Compatibility scope confirmed

References:

- `codex-rs/rollout/src/official_codex_discovery.rs:120`
- `codex-rs/rollout/src/official_codex_discovery.rs:171`
- `.pipeline/reviews/C8/M4/implement.md:19`
- `.pipeline/prompts/c8-004-session-discovery-strategy-official-codex.md:49`

The current compatibility implementation is scoped to a callable sanitized export API and manifest/artifact output. It does not implement automatic official Codex discovery, native import, or resume wiring in official Codex session listing. The compatibility claim should continue to be read as: sanitized export artifacts are produced and parse through the repo's rollout thread reader; automatic official Codex discovery/import remains outside this M4 implementation unless a later milestone wires it in.

## Validation Results

Executed from `/home/heyx/Codex-VSP/codex-rs`:

```sh
cargo test -p codex-rollout --test official_codex_discovery
```

Result: passed. 8 tests passed, 0 failed.

```sh
cargo fmt --check --package codex-rollout
```

Result: passed. The command emitted the existing repository warning that stable rustfmt cannot set unstable option `imports_granularity = Item`.

Executed from `/home/heyx/Codex-VSP`:

```sh
git diff --check
```

Result: passed with no output.

## Final Status

M4 audit recheck passes with one documented non-blocking residual risk around explicit source symlink hardening. The prior blocking findings for title leakage, duplicate basename overwrite, sanitized artifact parseability, and output symlink/raw-share are fixed and covered by focused tests.
