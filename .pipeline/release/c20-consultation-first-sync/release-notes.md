# C20 Consultation-First Sync/Doc Release Notes

Generated: 2026-06-30T21:54:08+08:00

Status: published as internal preview `v0.140.0-vsp.8.2`. The original local sync/doc note has been superseded by the versioned release notes at `.pipeline/release/v0.140.0-vsp.8.2/release-notes.md`.

## Scope

This sync/doc release note records the Codex-VSP target sync for the C20 consultation-first action boundary.

The synchronized behavior is:

- `discussion`, `background`, `idea`, `complaint`, `question`, and `solution-discussion` inputs are non-editing signals.
- Non-editing signals first receive a Mini-contract in this order: `我的理解` -> `问题原因` -> `推荐方案`.
- Clear imperative requests with a concrete target may proceed directly.
- After a plan, Mini-contract, or recommendation has been shown, replies such as `可以`, `确认`, `OK`, `go ahead`, or `apply it` authorize only the scope already shown to the user.

## Modified Surfaces

- Root Workflow/OpenCode guidance: `AGENTS.md`.
- OpenCode Workflow agents: `.opencode/agents/*.md`.
- OpenCode Workflow plan command adapters: `.opencode/commands/hw:plan:architecture.md` and `.opencode/commands/hw:plan:technical-stack.md`.
- Bundled model catalog runtime instructions: `codex-rs/models-manager/models.json`.
- Focused catalog regression test: `codex-rs/models-manager/src/manager_tests.rs`.
- User-facing docs: `CHANGELOG.md`, `docs/vsp-codex.md`, and `docs/slash_commands.md`.

## Runtime Path Evidence

- `codex-rs/core/src/config/mod.rs` loads explicit `base_instructions` first, then `model_instructions_file`, then legacy inline instructions.
- `codex-rs/core/src/session/mod.rs` resolves session base instructions from config override, conversation history, then current model metadata.
- `codex-rs/protocol/src/openai_models.rs` prefers `model_messages.instructions_template` when present.
- `codex-rs/models-manager/src/lib.rs` includes `models.json` as the bundled catalog, so both `base_instructions` and template fields need the boundary.

## Validation

- `python3 -m json.tool opencode.json .opencode/opencode.json codex-rs/models-manager/models.json`
- `node --check .opencode/runtime/hypo-workflow-hooks.js`
- `node --check .opencode/runtime/hypo-workflow-status.js`
- `node --check .opencode/plugins/hypo-workflow.ts`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-models-manager bundled_models -- --nocapture`
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`
- `git diff --check`

## Release Package

- Version: `0.140.0-vsp.8.2`.
- Release package: `.pipeline/release/v0.140.0-vsp.8.2/vsp-codex-preview-linux-x64-gnu.tar.gz`.
- SHA256: `427a101bfecfa123c6795c12d6a2ca4b5a26952b6f6b61ba9a654106e20cb65b`.
- Stable manifest first entry: `.pipeline/release/vsp-release-manifest.json` -> `0.140.0-vsp.8.2`.

## Non-Targets

- No runtime Rust behavior was changed for C20 consultation routing.
- No OpenCode plugin JavaScript behavior was changed.
- No `.pipeline/state.yaml`, `.pipeline/cycle.yaml`, or `.pipeline/rules.yaml` writes were made.

## Follow-Up Risk

The OpenCode file guard remains informational-only in `.opencode/runtime/hypo-workflow-hooks.js`. Turning that into a hard deny is a runtime permission behavior change and should be handled as a separate confirmed change.
