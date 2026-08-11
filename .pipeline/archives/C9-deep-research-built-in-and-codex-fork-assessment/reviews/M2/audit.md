# C9 M2 Audit Worker Report

## Verdict

`pass_with_risks`.

C9 M2 upstream merge is structurally acceptable to proceed to M3. This audit found no unresolved merge conflict paths, no strict conflict markers, and no whitespace or formatting failures. The recorded `cargo check --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex` result is sufficient as M2 structural compile evidence because it is the exact primary validation command for the merged VSP CLI surface and is recorded in Workflow authority files as passed with warnings only.

This worker did not modify source code, commit, push, tag, release, or delete `.pipeline/.lock`. The only report artifact written by this worker is `.pipeline/reviews/C9/M2/audit.md`.

## Commands

- `pwd && git branch --show-current && git status --short --branch`
  - Result: workspace is `/home/heyx/Codex-VSP`; branch is `c9-upstream-0.140.0-alpha.18`; worktree is a large dirty merge/update state as expected for M2.
- `git diff --name-only --diff-filter=U`
  - Result: passed; no unresolved merge conflict paths.
- `rg -n "^(<<<<<<<|>>>>>>>)" --glob '!target/**' --glob '!codex-rs/target/**'`
  - Result: passed; no strict conflict marker matches. `rg` exit code was 1, meaning no matches.
- `git diff --check`
  - Result: passed; no whitespace/conflict-marker diff hygiene failures.
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`
  - Result: passed. Output contained repeated stable rustfmt warnings: `can't set imports_granularity = Item`, but the command exited 0.
- `rg -n "unavailable in this build|not available in this build|Realtime.*unavailable|/realtime" codex-rs/tui/src --glob '!target/**'`
  - Result: confirmed `/realtime` TUI downgrade evidence at `codex-rs/tui/src/chatwidget/slash_dispatch.rs:265`.
- `sed -n '25,36p' .pipeline/PROGRESS.md && sed -n '7150,7172p' .pipeline/log.yaml`
  - Result: Workflow authority records state that `cargo check --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex` passed with warnings only, and record the `/realtime` downgrade caveat.

## Findings

- No unresolved merge conflicts remain.
  - Evidence: `git diff --name-only --diff-filter=U` produced no output and exited 0.
- No strict conflict markers remain outside target directories.
  - Evidence: strict `rg` scan for line-start `<<<<<<<` and `>>>>>>>` produced no matches.
- Diff hygiene and formatting are clean.
  - Evidence: `git diff --check` exited 0; `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check` exited 0.
- Structural compile evidence is acceptable for M2.
  - Evidence: `.pipeline/PROGRESS.md` and `.pipeline/log.yaml` record the primary command `cargo check --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex` as passed with warnings only. I did not rerun it in this audit pass because the recorded result is recent, exact, and sufficient for structural merge acceptance; M3/M5 should still broaden validation.
- VSP surface caveat is preserved and visible.
  - Evidence: `codex-rs/tui/src/chatwidget/slash_dispatch.rs:265` displays `Realtime voice is not available in this build.` for `/realtime`, while `/settings` realtime audio device selection code remains present around the same dispatch area.

## Risks

- `/realtime` is intentionally not functional in this merged TUI build. This is not a structural merge blocker for M2, but it is a user-facing regression risk if release notes or M3 validation imply realtime voice is restored.
- The `cargo check` evidence is package/binary scoped. It is appropriate for M2 structural compile confidence, but it does not prove all workspace crates, tests, app-server runtime paths, installer/package paths, snapshots, or realtime runtime behavior.
- The working tree remains very large and dirty because M2 is an upstream merge integration state. This audit only validates structural conflict/format/primary compile evidence; it does not review every semantic merge decision.

## Recommendation

Accept M2 as `pass_with_risks` and proceed to M3 `Preserve VSP Runtime Surfaces`.

M3 should explicitly validate the retained VSP surfaces, including Workflow Ask/request-user-input behavior, subagent/multi-agent surfaces, `vsp-codex` CLI/package surfaces, plugin sync surfaces, app-server protocol compatibility, and the documented `/realtime` downgrade. Do not treat realtime voice as release-ready until the runtime wiring is restored and covered by targeted tests.
