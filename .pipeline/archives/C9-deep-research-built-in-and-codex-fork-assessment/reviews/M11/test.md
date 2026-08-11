# C9 M11 Test Review - TUI CLI VSCode And User Surface Convergence

## Verdict

**BLOCKED.**

M11 implementation should not start as a broad merge until the current user-surface baseline failures are triaged. Focused Workflow slash routing and VSCode Remote install/status/rollback coverage pass, but request_user_input snapshot coverage, broad TUI nextest, and supplemental `vsp-codex` identity/update behavior expose blockers.

## Context Read

- `.pipeline/prompts/c9-011-tui-cli-vscode-and-user-surface-convergence.md`
- `.pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml`
- `.pipeline/reports/C9/M10-core-adapters.md`
- Read-only TUI/CLI context:
  - `codex-rs/tui/tests/suite/workflow_slash_routing.rs`
  - `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
  - `codex-rs/tui/src/chatwidget/plan_panel.rs`
  - `codex-rs/tui/src/chatwidget/goal_status.rs`
  - `codex-rs/cli/tests/vscode.rs`
  - `codex-rs/cli/tests/vsp_entrypoint.rs`
  - TUI/CLI snapshot inventory via read-only file listing/search.

M7 requires preserving VSP workflow routing, Question Tool/request_user_input semantics, VSP CLI identity, and VSCode Remote behavior. M10 reports core Workflow/runtime preservation as passed with follow-up risks around deeper goal lifecycle variants. Current dirty tree already contains unrelated `.pipeline`, core, state, and `codex-rs/tui/src/lib.rs` changes; this worker did not revert or edit them.

## Commands And Results

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui workflow_slash_routing -- --nocapture
```

Result: **PASS**. 9 workflow slash routing tests passed. Covered `/hw:*`, `$hypo-workflow:*`, unknown `/hw:*` routing, non-workflow command exclusion, Workflow command palette discoverability, and status summary claims.

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui request_user_input -- --nocapture
```

Result: **FAIL**. 88 passed, 1 failed. Failure:

- `bottom_pane::request_user_input::tests::request_user_input_freeform_remapped_interrupt_snapshot`
- Snapshot diff pads rendered blank/content lines to full terminal width while the accepted snapshot keeps compact lines. The footer line remains unchanged.
- Generated `.snap.new` was removed; no snapshot was accepted.

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture
```

Result: **PASS**. 16 tests passed. Covered VSCode settings scope selection, application-scope warnings, install/status dry run, server/user scope preference, remote bundled Codex executable shimming, running bundled ELF replacement, wrapper CODEX_HOME pinning, app-server initialize probe, and rollback scope behavior.

```bash
cargo nextest run --manifest-path codex-rs/Cargo.toml --no-fail-fast -p codex-tui
```

Result: **FAIL**. 3012 tests run: 2936 passed, 76 failed, 4 skipped.

Failure pattern:

- Multiple snapshot failures across request_user_input, footer, plan panel, status surface, markdown rendering, keymap, history cell, onboarding, and active subagent surfaces.
- Multiple TUI app tests aborted with stack overflow, including agent picker, feature-flag, side-thread, embedded app-server, and goal draft tests.
- Generated `.snap.new` files were removed; no snapshot was accepted.

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vsp_entrypoint -- --nocapture
```

Result: **FAIL**. 7 passed, 1 failed. Failure:

- `official_service_cli_commands_are_disabled`
- `vsp-codex update` in debug builds prints: `Error: \`vsp-codex update\` is not available in debug builds. Install a release build of VSP Codex to use this command.`
- Test expected the VSP official-service-disabled message: `This official Codex service is disabled in VSP Codex.`
- This is M11-relevant because the prompt explicitly names `vsp-codex` identity and updater behavior.

```bash
git diff --check
```

Result: **PASS**.

## Failure Summary

1. **Question Tool/request_user_input card rendering is not baseline-clean.** The focused request_user_input command fails on a freeform remapped interrupt snapshot. Because M11 requires snapshot updates to be semantic evidence, this cannot be mechanically accepted. The diff suggests a full-width padding/rendering behavior change in the Ask card body.

2. **Broad TUI regression is not baseline-clean.** nextest exposes widespread snapshot churn across plan panel, footer/status, markdown, keymap, history, status surfaces, and request_user_input, plus stack overflow aborts in app tests. Some stack overflow behavior resembles M10's heavy async test stack limitation, but these are TUI app tests and still block using broad TUI nextest as clean release evidence.

3. **`vsp-codex` update user surface conflicts with expected VSP identity policy.** VSCode behavior passes, but `vsp_entrypoint` shows `update` currently routes to a debug-build update message instead of the official-service-disabled VSP guard expected by the test.

## M11 Implementation Guardrails

- Preserve VSP `/hw:*` and `$hypo-workflow:*` routing. Focused coverage passes and should remain a no-regression gate.
- Preserve Question Tool/request_user_input card semantics before accepting any snapshot changes. Review width, padding, footer, keybinding, workflow confirmation, authorization-intent, and Chinese Workflow cards as user-facing behavior.
- Treat plan panel, status/footer, goal status, provider/setup, markdown/workflow rendering, keymap, and active subagent snapshot diffs as semantic UI evidence. Do not bulk accept snapshots.
- Preserve `vsp-codex` command identity and update behavior. The debug-build updater message needs policy triage against the VSP official-service-disabled expectation.
- Preserve VSCode Remote behavior. Current focused VSCode tests pass and should remain part of the M11 validation set.
- Do not use broad TUI nextest as PASS evidence until stack overflow aborts are either fixed or explicitly classified with a narrow harness/environment explanation.

## Snapshot And User Surface Risks

- Request card body padding changes can affect visual hierarchy and terminal layout, especially narrow/tight card cases.
- Plan panel snapshot failures are high risk for Workflow UX because M11 explicitly protects plan panel/status/footer surfaces.
- Status surface snapshot failures are high risk for provider/setup, permissions, model, usage limit, and remote status visibility.
- Markdown rendering snapshot failures can regress Workflow report/render readability.
- CLI update message mismatch can confuse VSP users about whether official OpenAI update/service commands are intentionally disabled or merely unavailable in debug builds.

## Allow Implement Worker To Start?

**No, not for broad M11 source convergence.**

Allow only a scoped triage/fix pass for the blockers above:

- Diagnose and fix or explicitly justify request_user_input full-width padding snapshot changes.
- Triage broad TUI snapshot failures by user surface, accepting only reviewed semantic changes.
- Resolve TUI stack overflow aborts or document a validated test harness limitation.
- Resolve `vsp-codex update` policy/message mismatch.

After that, rerun the focused commands plus broad TUI regression and `git diff --check`.

## Follow-up Validation - Plain Update Policy Sync

### Change

Updated `codex-rs/cli/tests/update.rs` so the debug-build plain `vsp-codex update` regression test now expects the VSP official-service-disabled guard instead of the old debug-build updater message. The test still preserves its original intent: plain `update` must fail before starting the interactive official update flow. It also asserts stderr does **not** contain `not available in debug builds`, so a regression back to the debug updater path is caught.

No production source, snapshots, reports, or Workflow state files were edited by this test worker in this follow-up.

### Follow-up Commands

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test update -- --nocapture
```

Result: **PASS**. 1 passed, 0 failed.

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vsp_entrypoint -- --nocapture
```

Result: **PASS**. 8 passed, 0 failed. Confirms `official_service_cli_commands_are_disabled` now passes with the implement worker's plain update guard behavior.

```bash
git diff --check
```

Result: **PASS**.

### Current M11 Baseline Status

**Focused baseline: PASS for the revalidated M11 surfaces reported by the main thread and this follow-up.**

Known focused evidence now includes:

- `codex-cli --test update`: PASS 1/1 after test policy sync.
- `codex-cli --test vsp_entrypoint`: PASS 8/8.
- Main-thread reported `codex-tui request_user_input`: PASS 89/89.
- Main-thread reported `codex-tui workflow_slash_routing`: PASS 9/9.
- Main-thread reported `codex-cli --test vscode`: PASS 16/16; prior `Text file busy` was classified as a transient environment issue.

Remaining caveat: the earlier broad `cargo nextest run -p codex-tui` failure remains historical evidence until a fresh broad TUI run is recorded. Snapshot updates must still be reviewed semantically, not mechanically accepted.
