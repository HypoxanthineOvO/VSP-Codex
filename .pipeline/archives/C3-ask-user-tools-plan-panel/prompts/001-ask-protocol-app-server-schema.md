# M1 Ask 协议模型与 app-server experimental schema

## Objective

把 `request_user_input` 扩展为可表达题型、总说明、逐题说明、结构化答案、secret 掩码和 native Ask capability 的跨客户端 experimental 契约。

## Source Of Truth

- P2: `.pipeline/plan-state/decompose.yaml` -> `M1`
- Deep Plan: `.pipeline/deep-plans/DP001-ask-user-tools/plan-context.md`
- Key files:
  - `codex-rs/protocol/src/request_user_input.rs`
  - `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
  - `codex-rs/app-server-protocol/src/protocol/v1.rs`
  - `codex-rs/app-server/src/bespoke_event_handling.rs`
  - `codex-rs/app-server-protocol/src/protocol/common.rs`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns failing/coverage tests before implementation where practical.
  - Add protocol/app-server tests for serde round-trip, schema export, experimental gating, native Ask capability, and legacy payload compatibility.
  - Must not implement production code.
  - Evidence: test names, expected failures before implementation when possible, final command output summary.
- `implement`
  - Owns protocol and app-server production changes only after test scope is clear.
  - Must preserve existing single-choice/legacy `answers: Vec<String>` compatibility.
  - Must not weaken app-server v2 payload conventions.
- `audit`
  - Checks final diff, tests, schema fixture drift, experimental/stable boundary, and worker separation evidence.
  - Must reject if `experimentalApi` is used as native UI support or if app-server Ask is accidentally stabilized.

## Technical Solution

Evolve the existing `request_user_input` stack instead of adding a parallel Ask API. Introduce explicit question kinds and structured answers in protocol/app-server types while keeping a compatibility path for existing string answer lists.

Native Ask UI support must be a separate client capability. `experimentalApi` only means the client accepts experimental protocol, not that it can render native Ask UI.

## Technical Route

1. Extend `codex-rs/protocol/src/request_user_input.rs`.
   - Add request-level summary/description.
   - Add explicit question kind.
   - Add per-question description.
   - Add option id.
   - Add structured answer fields for selected option ids, ordered option ids, free text, and legacy text answers.
   - Keep serde compatibility where possible.
2. Extend `codex-rs/app-server-protocol/src/protocol/v2/item.rs`.
   - Add matching `ToolRequestUserInput*` fields with camelCase wire names.
   - Keep v2 TS exports under `v2/`.
   - Continue treating this as experimental.
3. Add native Ask UI capability.
   - Place it in the existing initialize/client capability path using repo style.
   - Do not overload `experimentalApi`.
4. Update app-server conversions in `bespoke_event_handling.rs`.
   - Core event -> app-server params.
   - App-server response -> core response.
   - Preserve legacy answer conversion for current clients.
5. Regenerate schemas and update protocol tests.

## Research Status

Resolved. Existing local evidence:

- `ToolRequestUserInput*` lives in `codex-rs/app-server-protocol/src/protocol/v2/item.rs`.
- `item/tool/requestUserInput` lives in `protocol/common.rs`.
- `InitializeCapabilities` lives in `protocol/v1.rs`.
- Existing Ask RPC is documented as experimental.

## Risks And Alternatives

- Risk: replacing `answers: Vec<String>` outright breaks TUI/app-server paths. Mitigation: compatibility conversion.
- Risk: capability confusion with `experimentalApi`. Mitigation: native Ask capability is distinct.
- Rejected alternative: new ask_user tool family. Reason: existing `request_user_input` stack already spans protocol/core/app-server/TUI.

## Validation Path

Run from `codex-rs/` unless the root `justfile` handles cwd:

- `just write-app-server-schema`
- `just write-app-server-schema --experimental`
- `just test -p codex-app-server-protocol`

Pass signal:

- New experimental Ask fields and native Ask capability appear in schema/TS fixtures.
- Legacy request/response payloads still deserialize or convert.
- App-server protocol tests pass.

Fail signal:

- Schema misses new fields.
- Ask surface is accidentally stable.
- Legacy single-choice payload breaks.

## Completion Criteria

- Protocol/app-server types support C3 Ask semantics.
- Native Ask capability exists and is separate from `experimentalApi`.
- Schema fixtures are updated.
- M1 report is written to `.pipeline/reports/001-ask-protocol-app-server-schema.md`.
