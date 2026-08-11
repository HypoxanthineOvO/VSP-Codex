# M12 Subagent 与 Plan UI 一致性修复

## Objective

修复 M11 后暴露的两个 UI 一致性问题：

1. Subagent spawn 完成提示仍显示 raw thread id，例如 `Spawned 019e67be-... (gpt-5.5 medium)`，而 active Subagent 列表已经显示昵称；spawn 提示、列表、详情应统一显示昵称和工作类型。
2. Plan 折叠态在 milestone 完成后仍显示旧步骤，例如 `Plan: 运行 focused TUI 测试并处理 snapshot (2/4)`；完成前必须同步最终 `update_plan` 状态，避免常驻 Plan 面板停在旧 checklist。

## Source Of Truth

- 用户在 M11 完成后反馈：spawn 提示应为 `派出 Subagent 李宇轩 [...] (gpt-5.5 medium)`，并且 Subagent 列表也应显示 Tester/Implement/Audit 等工作类型。
- 用户反馈：M11 完成后 Plan 折叠态仍显示旧的 `运行 focused TUI 测试并处理 snapshot (2/4)`。
- 本地定位：
  - Subagent history cell 渲染在 `codex-rs/tui/src/multi_agents.rs`。
  - Active Subagent panel 在 `codex-rs/tui/src/chatwidget/active_subagents.rs`。
  - metadata 同步入口在 `ChatWidget::set_collab_agent_metadata`。
  - Plan panel 折叠渲染在 `codex-rs/tui/src/chatwidget/plan_panel.rs`。
  - Plan panel 只消费最近一次 `update_plan`/PlanState，不会自动读取 `.pipeline/state.yaml`。

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns red tests and snapshots before implementation.
  - Scope:
    - `codex-rs/tui/src/multi_agents.rs` tests/snapshots.
    - `codex-rs/tui/src/chatwidget/tests/app_server.rs` active Subagent panel snapshots.
    - `codex-rs/tui/src/chatwidget/plan_panel.rs` tests/snapshots.
    - Optional core/session test if final-before-response Plan guard is implemented outside TUI.
  - Must prove current behavior fails before implementation where feasible.
  - Expected artifacts:
    - `.pipeline/reviews/C3/M12/test.md`

- `implement`
  - Owns production/runtime edits only.
  - Allowed likely paths:
    - `codex-rs/tui/src/multi_agents.rs`
    - `codex-rs/tui/src/chatwidget/active_subagents.rs`
    - `codex-rs/tui/src/chatwidget/plan_panel.rs`
    - `codex-rs/tui/src/chatwidget/protocol.rs` or adjacent Plan notification handling if needed.
    - Core/session/tool reminder files only if Plan final guard cannot be solved in TUI/workflow layer.
  - Must not edit tests/snapshots/review artifacts.
  - Expected artifacts:
    - `.pipeline/reviews/C3/M12/implement.md`

- `audit`
  - Read-only final review.
  - Checks nickname/role consistency, no raw id fallback when metadata is available, role label quality, Plan completion semantics, snapshot quality, and no regressions in English/default flows.
  - Expected artifacts:
    - `.pipeline/reviews/C3/M12/audit.md`

- Main agent
  - Orchestrates workers, integrates changes, accepts intended snapshots, updates `.pipeline`, runs validation, and builds `vsp-codex` if UI behavior changes.
  - Must not satisfy both test and implement roles locally while worker separation is strict unless a worker is blocked and the lifecycle record explains the fallback.

## Technical Solution

### Subagent display

Use a single display label contract for Subagent surfaces:

- Preferred label: `<nickname> [<role>]`
- Fallback when role missing: `<nickname>`
- Fallback when nickname missing: raw thread id or `Subagent`
- Spawn completed title:
  - zh-CN/VSP preferred: `派出 Subagent <label> (<model> <reasoning>)`
  - Existing English paths may keep `Spawned <label> (...)` if localization is not globally enabled.
- Active Subagent compact/detail rows should include the same `<nickname> [<role>]` label.

Role display:

- Use `agent_role` from metadata when present.
- If role is specific (`test`, `implement`, `audit`, `worker`, `explorer`, etc.), display it.
- If role can be mapped from workflow wording or tool metadata, prefer specific `test`/`implement`/`audit`; otherwise show the raw role instead of hiding it.

### Plan completion sync

Plan panel state is driven by `update_plan`, not `.pipeline/state.yaml`. Therefore the fix needs both a behavioral rule and UI resilience:

- Workflow rule: before final response after a milestone/report completion, call `update_plan` with final statuses.
- Guard option: if Plan freshness is stale before a final response, inject or enforce a final update reminder rather than only after tool bursts.
- TUI resilience: when all items are completed, collapsed Plan should render a completion summary instead of an arbitrary previous active item.

M12 should implement the smallest robust combination:

1. Add tests showing Plan collapsed completion renders clearly when all items are completed.
2. Add or document lifecycle enforcement that final milestone closeout updates `update_plan`.
3. If possible, add a core/session guard so stale Plan cannot be silently left stale before final answer.

## Technical Route

1. Audit current Subagent render path:
   - `multi_agents::spawn_end`
   - `title_with_agent`
   - `agent_label_spans`
   - `active_subagent_panel_lines`
   - active detail popup item construction
   - `ChatWidget::set_collab_agent_metadata`

2. Write red coverage:
   - Spawn completed with metadata renders nickname + role instead of raw UUID.
   - Active Subagent compact panel renders role next to nickname.
   - Active Subagent detail popup renders role next to nickname.
   - If metadata arrives late, active row refreshes nickname + role.
   - Plan collapsed all-completed state renders completed summary.

3. Implement label unification:
   - Add a helper local to TUI multi-agent display if existing helpers are insufficient.
   - Prefer `agent_nickname` and `agent_role`.
   - Preserve raw id fallback only when nickname unavailable.
   - Keep model/reasoning span unchanged.

4. Implement Plan collapsed completion behavior:
   - In `plan_panel.rs`, if `progress_counts(state)` is all completed, render a completion-focused line.
   - Avoid changing expanded rendering unless needed.
   - Ensure active-root selection does not pick a stale in-progress item after all items are complete.

5. Implement final Plan sync guard if technically feasible in focused scope:
   - Review M10 Plan freshness implementation.
   - Add a reminder/guard for final response when active Plan is stale.
   - If core guard is too broad for M12, explicitly record workflow-level requirement and add a regression test for TUI completed state.

6. Validate and rebuild:
   - Run focused TUI tests/snapshots.
   - Run `just fmt`.
   - Run `just fix -p codex-tui`.
   - Rebuild `cargo build --bin vsp-codex` for real TUI verification.

## Research Required

- status: resolved
  evidence: "`multi_agents.rs` already has `AgentMetadata { agent_nickname, agent_role }`, `agent_label_spans`, and spawn history cell rendering. `active_subagents.rs` stores `nickname` and `role` but current user-visible row behavior is insufficient. `plan_panel.rs` collapsed rendering uses latest `PlanState`, progress counts, and active root/subplan selection; it does not read `.pipeline/state.yaml`."

## Risks And Alternatives

- Risk: spawn completion event can arrive before metadata. Mitigation: use cached metadata when available, keep fallback raw id only when nickname is unavailable, and preserve late metadata refresh for active panel.
- Risk: role value is generic `worker`. Mitigation: display raw role rather than hiding it; map to `test`/`implement`/`audit` only when reliable.
- Risk: forcing Plan update before every final response may create noisy model behavior. Mitigation: trigger only when active Plan exists and is stale or when a milestone/report completion is being finalized.
- Risk: TUI all-completed collapse is only a resilience fix and does not update stale Plan source. Mitigation: pair with workflow/core final sync rule.
- Alternative rejected: make Plan panel read `.pipeline/state.yaml`. This would couple TUI runtime state to Hypo-Workflow files and not help general Codex sessions.

## Validation Path

Automated:

- `cd codex-rs && just test -p codex-tui collab_agent_transcript`
- `cd codex-rs && just test -p codex-tui app_server_collab_spawn_completed_renders_requested_model_and_effort active_subagents`
- `cd codex-rs && just test -p codex-tui plan_panel`
- `cd codex-rs && cargo insta pending-snapshots --workspace`
- `cd codex-rs && just fmt`
- `cd codex-rs && just fix -p codex-tui`
- `cd codex-rs && cargo build --bin vsp-codex`

Manual:

- Spawn a test/implement/audit Subagent in real TUI.
- Confirm the immediate history line shows nickname + role, not raw UUID when nickname is known.
- Confirm the active Subagent compact list and detail popup show nickname + role.
- Complete a task with a visible Plan.
- Confirm collapsed Plan does not remain on an old in-progress checklist item.

Pass signal:

- Spawn history and active Subagent panel use consistent identity labels.
- Role is visible in Subagent list/detail.
- Plan collapsed state shows completed/waiting-final-gate status after closeout.
- No pending snapshots.

Fail signal:

- Spawn history still shows UUID despite metadata being available.
- Active Subagent list hides role.
- Plan collapsed state remains stuck on an old in-progress item after final closeout.

## Audit Focus

- Verify raw thread ids are only fallback, not primary display when nickname exists.
- Verify role display does not invent incorrect test/implement/audit labels.
- Verify Plan completion rendering does not hide real pending items.
- Verify final Plan sync behavior does not spam update_plan reminders in normal turns.
- Verify snapshots cover both history line and active panel surfaces.
