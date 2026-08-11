# M2 Core Ask 工具语义与模型契约

## Objective

让 root agent 在普通交互 turn/default bypass 中使用 Ask，并支持“问题 + 简述 + 回答模块”、多题型 normalization、secret 策略和模型防滥用说明。

## Source Of Truth

- P2: `.pipeline/plan-state/decompose.yaml` -> `M2`
- Depends on M1 protocol/app-server types.
- Key files:
  - `codex-rs/core/src/tools/handlers/request_user_input_spec.rs`
  - `codex-rs/core/src/tools/handlers/request_user_input.rs`
  - `codex-rs/core/src/session/mod.rs`
  - `codex-rs/core/src/session/tests.rs`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns tests for schema/normalization, default bypass availability, root-only rejection, and pending-turn behavior.
  - Must check that no per-turn/per-request hard limit is introduced.
- `implement`
  - Owns core tool spec, normalization, and mode availability changes.
  - Must keep root-thread-only behavior.
  - Must not implement TUI rendering; that belongs to M3.
- `audit`
  - Checks model-facing wording, availability modes, subagent boundary, and UI-less/fallback behavior.
  - Must reject if the tool is Plan-only or if subagents can directly prompt the user.

## Technical Solution

Keep `RequestUserInputHandler` and `session.request_user_input`. Change schema and normalization so question kind drives validation instead of inferring from `options`. Ask is a general collaboration primitive for ordinary interactive turns, Confirm Gates, acceptance, and normal clarification.

Do not add hard limits like “one Ask per turn” or “max N questions”. Use model guidance to discourage unnecessary questions.

## Technical Route

1. Update `request_user_input_spec.rs`.
   - Add request-level summary/description.
   - Add question kind.
   - Add per-question description.
   - Allow no options for short/free text.
   - Allow multi-select and ranking option sets.
2. Rewrite `normalize_request_user_input_args`.
   - Validate required fields by kind.
   - Preserve compatible defaults for old option-only calls.
   - Do not auto-force every question into old `is_other` semantics if that conflicts with kind.
3. Ensure VSP/default bypass mode availability.
   - Use existing feature/config path where possible.
   - Keep root-thread-only guard in `request_user_input.rs`.
4. Update core/session tests.
   - Existing pending request_user_input test should still pass.
   - Add coverage for default mode, subagent rejection, and free/ranking validation.

## Research Status

Resolved. Current code facts:

- `request_user_input_spec.rs` forces non-empty options and auto sets `is_other=true`.
- `request_user_input.rs` already checks root-thread-only and mode availability.
- Core has `pending_request_user_input_does_not_spawn_extra_goal_continuation`.

## Risks And Alternatives

- Risk: model overuses Ask. Mitigation: strong tool description, no hard user-opposed limits.
- Risk: too broad availability causes UI-less waits. Mitigation: explicit fallback/fail path in unsupported contexts.
- Rejected alternative: Plan-only Ask. User explicitly rejected it.

## Validation Path

- `just test -p codex-core request_user_input`
- `just test -p codex-core pending_request_user_input_does_not_spawn_extra_goal_continuation`

Pass signal:

- Question-kind normalization works.
- Default bypass/root-only behavior is correct.
- Pending Ask keeps turn open without spurious continuation.

## Completion Criteria

- Core Ask schema supports all M2 kinds.
- No hard per-turn/question limit is added.
- Root-only and ordinary interactive behavior are covered.
- M2 report is written to `.pipeline/reports/002-core-ask-tool-semantics.md`.
