# C10 M3 - Stage-2 Merge To rust-v0.143.0-alpha.33 And VSP Surface Recheck

## Objective

Merge `rust-v0.143.0-alpha.25..rust-v0.143.0-alpha.33` (70 commits, ~419 files) and re-verify every VSP must-preserve surface, with special attention to upstream defaults that conflict with VSP policy.

## Requirements

- `git merge rust-v0.143.0-alpha.33` (commit `5395e1494c`) on the C10 branch.
- Hot domains from the delta: app-server (environment info RPC, read timeouts, structured shutdown logs), plugins (marketplace source policy, remote plugins enabled by default), protocol (rollout turn items), core (synthesized call output IDs, custom tool namespaces), multi-agent v2 / Rendezvous WebSocket lifecycle.
- Upstream "enable remote plugins by default" must yield to VSP disabled-remote policy; verify `codex-core-plugins` disabled-remote behavior after merge.
- multi-agent v2 / Rendezvous WebSocket: follow upstream internally; do NOT expose as VSP-committed capabilities.
- Recheck the full must_preserve list from C9: VSP provider/official-service disablement, `vsp-codex` CLI naming + VSP release manifest updater, VSCode Remote SSH install/status/rollback/repair, Hypo-Workflow /hw routing + request_user_input metadata + Workflow markdown + plan freshness, Subagent UX, plugin/skills compatibility under VSP source rules, sandbox/permissions without changing VSP safety defaults. voice/realtime and remote-control remain non-user-facing.

## Boundaries

- In scope: Stage-2 merge, conflict resolution, VSP policy re-assertion edits, schema regeneration if protocol changed.
- Out of scope: version/release edits, extension capability gap fixes beyond merge fallout (M4 owns those).
- Fallback: uncontrollable conflicts -> stop and report via Question Tool.

## Technical Solution

Smaller, newer delta merged on top of a validated alpha.25 base. Policy conflicts (remote plugins default) are resolved by explicit VSP-side configuration/code assertions with regression tests, not by dropping upstream code.

## Technical Route

1. Merge alpha.33; record conflicts to `.pipeline/reports/C10/M3-merge-conflicts.md`.
2. Resolve by hot domain; regenerate schemas if protocol crates changed.
3. Surface-by-surface must_preserve recheck with a written checklist in `.pipeline/reports/C10/M3-vsp-surface-recheck.md`.
4. Focused suites + plugins disabled-remote tests + entrypoint tests.

## Research Required

Evaluate during merge: actual impact of multi-agent v2 and Rendezvous WebSocket changes on VSP runtime (known trigger: new upstream platform capability). If impact is unclear or destructive, report before resolving.

## Risks And Alternatives

- Risk: upstream core refactors break Hypo-Workflow routing/goals runtime; fallback is stop-and-ask, never silent hard-force.
- Risk: policy defaults flipped silently by upstream config plumbing; mitigation is explicit tests for disabled-remote and official-service disablement.

## Validation Commands

```bash
git merge HEAD --is-ancestor rust-v0.143.0-alpha.33 && echo merged
cargo fmt --manifest-path codex-rs/Cargo.toml --all -- --check
cargo check --manifest-path codex-rs/Cargo.toml --workspace
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core-plugins
cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-protocol
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui workflow_slash_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vsp_entrypoint -- --nocapture
git diff --check
```

## Evidence

Pass signals: alpha.33 is an ancestor of HEAD, all listed suites green, surface recheck checklist complete with per-surface evidence, remote-plugins default verified disabled under VSP policy.

Fail signals: any must_preserve surface regressed; remote plugins or official service enabled by default; voice/realtime or remote-control exposed as VSP capabilities.

## Subworker Assignment Plan

- `test`: owns the surface-recheck test evidence and final suite runs; writes `.pipeline/reviews/C10/M3/test.md`. Rejects checklists asserted without executed evidence.
- `implement`: owns merge resolution and VSP policy re-assertion edits. No test asset edits.
- `audit`: read-only diff/policy/lifecycle review; writes `.pipeline/reviews/C10/M3/audit.md`.
- Main agent: orchestration and lifecycle only.
- Non-overlap: distinct identities for the three roles.

## Expected Artifacts

- Merge commit(s) reaching alpha.33
- `.pipeline/reports/C10/M3-merge-conflicts.md`
- `.pipeline/reports/C10/M3-vsp-surface-recheck.md`
- `.pipeline/reviews/C10/M3/test.md`
- `.pipeline/reviews/C10/M3/audit.md`
