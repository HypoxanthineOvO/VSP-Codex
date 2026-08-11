# M3 Reasoning Capability Gate

## Objective

确认并接入真实 Thinking/Reasoning 输出；如果 raw/summary 不可得，明确降级而不伪造。

## Source Of Truth

- `.plan-state/decompose.yaml` M3
- Existing app-server v2 notifications:
  - `item/reasoning/summaryTextDelta`
  - `item/reasoning/summaryPartAdded`
  - `item/reasoning/textDelta`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns core/app-server/TUI reasoning tests and unsupported/no-empty-block snapshots.
  - Expected artifact: `.pipeline/reviews/C4/M3/test.md`
- `implement`
  - Owns event capability detection, config compatibility, and TUI collapsed Thinking presentation.
  - Expected artifact: `.pipeline/reviews/C4/M3/implement.md`
- `audit`
  - Checks no fake reasoning, correct v2 mapping for VSCode, and old config compatibility.
  - Expected artifact: `.pipeline/reviews/C4/M3/audit.md`

## Technical Solution

Reasoning display is capability-driven. Use real core reasoning events and existing app-server v2 mappings. TUI groups real summary/raw/section break content into a collapsed Thinking block. No provider event means no block.

## Technical Route

1. Inspect core Responses event handling for summary/raw/section reasoning events.
2. Add or update tests proving app-server v2 maps summary/raw deltas correctly.
3. Add TUI collapsed Thinking block for real reasoning items only.
4. Add three-state visibility: `off / summary / raw`.
5. Map old `show_raw_agent_reasoning=true` to raw and `false` to summary.
6. If `ConfigToml` changes, run `just write-config-schema`.
7. Add snapshots: summary, raw, off hidden, unsupported no empty block, section break.

## Research Required

Partially resolved: protocol and app-server surfaces exist. Implementation must verify what the active VSP provider actually emits.

## Risks And Alternatives

- Risk: raw reasoning is unavailable. Accept summary or unsupported verdict.
- Risk: tool/progress output is mislabeled as Thinking. Reject this.
- Alternative: only release-note verdict if TUI block proves too risky, but capability judgment remains required.

## Validation Path

Commands:

- `just test -p codex-core reasoning`
- `just test -p codex-app-server-protocol reasoning`
- `just test -p codex-tui reasoning`

Pass signal: real summary/raw maps to app-server/TUI; off hides; unsupported shows no empty block or error.

Fail signal: fake Thinking appears, legacy raw-only path is used for VSCode, or config compatibility breaks.

## Audit Focus

- No synthetic hidden reasoning.
- app-server v2 remains VSCode compatibility path.
- Raw mode is explicit and backward compatible.
