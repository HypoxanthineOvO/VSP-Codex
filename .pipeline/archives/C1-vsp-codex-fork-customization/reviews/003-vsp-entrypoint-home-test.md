# Milestone 3 Test Worker Report

## Scope

Role: test worker only.

Allowed write paths touched:

- `codex-rs/cli/tests/vsp_entrypoint.rs`
- `codex-rs/utils/home-dir/src/lib.rs` test module only
- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/app/tests/vsp_home_import.rs`
- `.pipeline/reviews/003-vsp-entrypoint-home-test.md`

No production behavior, manifests, wrappers, scripts, Justfile, Bazel files, or implement/audit review files were edited.

## Test Changes

- Added focused CLI regression tests that resolve `codex_utils_cargo_bin::cargo_bin("vsp-codex")`, assert `vsp-codex --version` works, and assert top-level help usage is user-visible as `vsp-codex`.
- Updated the home-dir default test to expect `~/.vsp-codex` when `CODEX_HOME` is unset. Existing `CODEX_HOME` missing/file/canonicalize tests remain in place.
- Registered a new TUI app test module for VSP home import.
- Added TUI contract tests for a future `crate::vsp_home_import` API:
  - prompt/import plan appears only when default VSP config is missing, sibling legacy `~/.codex/config.toml` exists, and `CODEX_HOME` is unset;
  - no prompt with `CODEX_HOME` override;
  - no prompt when VSP config already exists;
  - no prompt when legacy config is missing;
  - import copies legacy home content into VSP home without overwriting existing VSP files;
  - prompt rendering has an `insta` snapshot test hook.

Snapshot note: the prompt rendering helper does not exist yet, so no `.snap` was generated or accepted. The snapshot assertion is present as an implementation contract and should generate under `codex-rs/tui/src/app/snapshots/` once the helper exists.

## Commands And Results

### `just fmt`

Result: passed.

Evidence: rustfmt reported `64 files left unchanged`; no formatter failures.

### `just test -p codex-utils-home-dir`

Result: failed as expected while production default still points to legacy home.

Evidence:

- 4 tests run: 3 passed, 1 failed.
- Failing test: `tests::find_codex_home_without_env_uses_vsp_default_home_dir`.
- Red reason:

```text
Diff < left / right > :
 AbsolutePathBuf(
<    "/home/heyx/.codex",
>    "/home/heyx/.vsp-codex",
 )
```

This confirms `CODEX_HOME` override and canonicalize behavior still pass, while the new default-home requirement is red.

### `just test -p codex-cli vsp_entrypoint`

Result: invalid local invocation, superseded by the next focused command.

Evidence: nextest treated `vsp_entrypoint` as a test-name filter and reported `0 tests run`.

### `just test -p codex-cli --test vsp_entrypoint`

Result: failed as expected while the Cargo binary is still named `codex`.

Evidence:

- 2 tests run: 0 passed, 2 failed.
- Failing tests:
  - `vsp_codex_version_entrypoint_is_available`
  - `vsp_codex_help_uses_vsp_codex_usage`
- Red reason:

```text
Error: could not locate binary "vsp-codex"; tried env vars ["CARGO_BIN_EXE_vsp-codex", "CARGO_BIN_EXE_vsp_codex"]; assert_cmd fallback failed: Cause: Cargo command not found: /home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex
```

### `just test -p codex-cli`

Result: failed only on the new VSP entrypoint tests.

Evidence:

- 219 tests run: 217 passed, 2 failed.
- Failing tests were the same two `codex-cli::vsp_entrypoint` tests.
- Red reason: missing `target/debug/vsp-codex` binary.

### `just test -p codex-tui vsp_home_import`

Result: compile-failed as expected while production import API/module does not exist.

Evidence:

- Compile error: `E0432 unresolved import crate::vsp_home_import`.
- Missing contract items:
  - `CodexHomeEnv`
  - `LegacyHomeImportPlan`
  - `LegacyHomeImportProbe`
  - `plan_legacy_home_import`
  - `copy_legacy_home_into_vsp_home`
  - `render_legacy_home_import_prompt_plain_text`

The compile failure intentionally records the implementation contract for the TUI first-run legacy home import behavior.

## Current Red State

- CLI red: no `vsp-codex` external binary target exists yet.
- Home-dir red: default unset env path is still `~/.codex`.
- TUI red: `crate::vsp_home_import` module/API does not exist yet.
- Snapshot pending: prompt snapshot will be generated once `render_legacy_home_import_prompt_plain_text` exists.
