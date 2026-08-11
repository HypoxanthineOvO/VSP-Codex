# M4 Pending Ask resume 与 unsupported-client fallback

## Objective

保证 pending Ask 跨设备 resume 能恢复 native UI；当客户端不支持 native Ask UI 时降级为普通编号聊天问题，不能卡死。

## Source Of Truth

- P2: `.pipeline/plan-state/decompose.yaml` -> `M4`
- Depends on M1 native Ask capability and M2/M3 Ask semantics.
- Key files:
  - `codex-rs/tui/src/app/pending_interactive_replay.rs`
  - `codex-rs/tui/src/app/app_server_requests.rs`
  - `codex-rs/app-server/src/thread_state.rs`
  - `codex-rs/app-server/src/bespoke_event_handling.rs`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns tests for pending replay metadata, resolved clearing, multi-prompt FIFO, native Ask replay, and unsupported fallback.
  - Must document that local tests do not replace final real cross-device acceptance.
- `implement`
  - Owns pending metadata, capability routing, fallback prompt/mapping, and fallback answer handling.
  - Must not block indefinitely when native Ask UI is unavailable.
- `audit`
  - Checks deadlock paths, capability branching, stale prompt replay, and final acceptance gate.
  - Must reject if `experimentalApi` is treated as native UI support.

## Technical Solution

Extend existing pending interactive replay so `RequestUserInput` stores enough question metadata for replay/fallback. Native-capable clients receive the original Ask request. Unsupported clients receive a normal assistant message using the numbered fallback template.

If the original tool call cannot be resumed, explicitly degrade to normal conversation and continue with the answer as new-turn context.

## Fallback Template

```text
我需要你回答几个问题后继续。当前客户端不支持原生 Ask UI，请按编号回复：

1. [short_answer] 你的问题文本
2. [single_choice] 请选择一个：
   A. 选项 label - description
   B. 选项 label - description
3. [ranking] 请按优先级排序：A, B, C

请用类似下面格式回答：
1. ...
2. A
3. B, A, C
```

Keep internal mapping: `call_id`, `turn_id`, question ids, kinds, option ids, and fallback letter -> option id.

## Technical Route

1. Extend pending interactive state.
   - Store Ask metadata, not only turn/item id.
   - Preserve FIFO behavior for multiple prompts in one turn.
2. Store/read native Ask client capability.
   - Use capability from M1.
   - Route replay/fallback based on actual UI support.
3. Implement fallback prompt builder and mapping.
4. Implement fallback answer parser or model handoff.
   - Parse strict numbered replies when possible.
   - Ask clarification when parse fails.
5. Add tests for stale/resolved prompts and unsupported clients.

## Research Status

Resolved. Current `pending_interactive_replay.rs` tracks request_user_input ids and has tests, but pending request metadata only stores turn id and item id.

## Risks And Alternatives

- Risk: original oneshot receiver no longer exists. Mitigation: explicit normal-chat downgrade path.
- Risk: fallback parsing is brittle. Mitigation: strict numbered template and clarification retry.
- Risk: local tests miss real cross-device behavior. Mitigation: final C3 manual acceptance gate.

## Validation Path

- `just test -p codex-tui pending_interactive_replay`
- `just test -p codex-app-server`
- `just test -p codex-core request_user_input`

Pass signal:

- Native-capable clients replay pending Ask.
- Unsupported clients do not wait forever and receive fallback.
- Resolved prompts do not replay.

## Completion Criteria

- Pending Ask is recoverable or explicitly downgraded.
- No known deadlock path remains.
- M4 report is written to `.pipeline/reports/004-pending-ask-resume-fallback.md`.
