---
kind: stone-result
cycle: C28-prompt-stack-quality-redesign
stone: S4
status: accepted
updated: 2026-08-10T22:24:25+08:00
source_edits: locked
---

# S4 Result: AGENTS, Environment, and External Injection Boundaries

Accepted by the user on 2026-08-10. Acceptance advances C28 to S5 and does not authorize product source edits until S5 itself is accepted.

## Conclusion

S4 recommends replacing the always-on 25,981-character root AGENTS body with a <=4K universal root and path-scoped Rust/Test, TUI, app-server/protocol, leaf, and Python rule files. This is a scope correction, not rule deletion. It also removes stale OpenCode/Hypo adapter instructions from Codex-visible repository rules, keeps environment facts while moving them to developer role, and keeps external Workflow/plugin injections short, conditional, and owner-isolated.

## Technical approach

1. Root AGENTS owns only universal repository rules, target-path instruction routing, and platform-wide constraints.
2. Domain files own their detailed rules. Before mutation, instructions from repository root through every target path must be loaded in order.
3. S5 chooses the smallest reliable enforcement mechanism. A runtime preflight is preferred if model-only routing cannot prove coverage; arbitrary shell text must not be parsed as a path oracle.
4. Repository/global AGENTS remain user-role because users and repository owners author them. Host-generated environment facts move to developer-role world state.
5. Installed Hypo-Workflow Skills own lifecycle semantics. VSP keeps only a short developer-role host boundary and trusted current-focus projection.

## Scope map

| Rule owner | Target surface |
|---|---|
| Universal repository | root `AGENTS.md`, <=4K |
| Rust/build/test | `codex-rs/AGENTS.md` |
| TUI | `codex-rs/tui/AGENTS.md` |
| TUI leaf state machine | existing `codex-rs/tui/src/bottom_pane/AGENTS.md` |
| App-server v2 | `codex-rs/app-server/AGENTS.md` |
| Protocol wire shapes | `codex-rs/app-server-protocol/AGENTS.md` from the same source fragment |
| Python | relevant Python roots |
| Workflow lifecycle | installed Hypo-Workflow Skills, not repository AGENTS |

## Evidence and validation

- M1 user-context: 26,472 characters; root AGENTS body: 25,981 characters, approximately 98% of the layer.
- Section sizes: Tests 6,214; app-server 3,688; TUI 2,487; Workflow/platform adapter sections approximately 5.5K.
- Source inspection confirms the loader walks only project root -> session cwd and applies a root-first 32 KiB aggregate budget.
- A root-cwd session therefore does not automatically see the existing bottom-pane AGENTS when later editing that path.
- `.pipeline/state.yaml` remains completed C10 legacy state; `.opencode/hypo-workflow.json.analysis` does not exist; actual analysis settings are nested in `.opencode/hypo-workflow.json` and are hybrid/confirm.
- Installed plugin v15 explicitly protects legacy state files and uses semantic index/Cycle records, proving the repository OpenCode adapter has drifted.
- Workflow Markdown/YAML and diff whitespace checks are required before S4 acceptance; no product tests apply before implementation.

## Environment behavior

Keep `cwd`, selected environment, shell, current date/timezone, workspace roots, structured filesystem profile, and active subagent identities. They are small and operationally useful. Inject once and replace only on relevant world-state change. Do not repeat prose permission policy inside this block.

Moving the environment block to developer role is intentional: it is host state, not user intent. M3 must compare real rollout ordering and cache behavior, including remote/multi-environment turns, before the change is considered safe.

## External boundaries

- `<hypo_workflow_host>` remains conditional developer text and does not embed lifecycle procedures.
- Active Cycle/focus messages use trusted plugin projection. Unknown focus is non-blocking and must not contradict the semantic index.
- External Skill bodies stay trigger-only, source-marked user fragments; they cannot override higher-role safety or expand user authorization.
- Apps/Plugins metadata follows S3's conditional developer-role design.
- Compaction/resume restores each current world-state section once and replaces stale sections by stable section ID.

## Expected result

- Ordinary root questions receive <=4K repository rules instead of 25,981 characters, at least 84% less AGENTS baseline.
- TUI, app-server and test details appear only in applicable work.
- Codex no longer receives unavailable OpenCode tool names, stale C10 write duties, or a nonexistent analysis path.
- Targeted code work still receives every relevant rule before mutation.
- Environment and external plugin state have clear role and ownership boundaries.

## M2/M3 test design

- Universal root-only prompt for non-code/root tasks.
- Ordered root -> language -> domain -> leaf resolution for local, remote, multi-root, and Windows paths.
- Mutation coverage/preflight, including `apply_patch` and the explicitly supported mutation surfaces.
- Specific rules are not starved by the aggregate byte budget; truncation is observable.
- AGENTS replacement/removal across turn, compaction, resume, fork, and selected-environment change.
- Developer-role environment ordering, XML escaping, diff-only updates, caching, and tool environment IDs.
- Conditional Workflow host/focus projection, unbound non-blocking behavior, and no external business-prompt rewrite.

## Problems and risks

- Domain splitting increases upstream merge structure differences. S5 should stage stale-adapter removal and the three large stable domains first.
- Model-only path routing may be insufficient; runtime preflight is safer but increases implementation scope and must not pretend to understand arbitrary shell scripts.
- Environment role migration may alter provider caching. Real before/after rollout evidence is mandatory.
- Plugin adapter drift is externally owned. C28 fixes the VSP-visible boundary but does not silently modify installed plugin packages.

## Modified files in S4

Only C28 Workflow review/result/progress artifacts were changed. Product `AGENTS.md`, Rust source, plugin cache, protected `.pipeline/state.yaml`, and configuration remain untouched.
