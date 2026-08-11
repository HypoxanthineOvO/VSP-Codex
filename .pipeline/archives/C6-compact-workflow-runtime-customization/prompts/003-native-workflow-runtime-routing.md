# M3 Native Workflow Runtime Routing

## Objective

让 VSP-Codex 在检测到 `.pipeline/` 项目时默认具备 Hypo-Workflow 内建行为，而不是只有用户显式提到 Skill 时才执行 Workflow 规则。

## Source Of Truth

- P1 用户目标：默认集成 Hypo-Workflow Skill 为系统内部能力，特化 Subagent、Question 和 Plan 能力适配。
- P2 技术路线：Workflow project detector、runtime instruction contributor、`/hw:*` slash routing、context-aware Skill injection、自动 Subagent 使用提示。
- 代码证据：
  - 初始上下文构造：`Session::build_initial_context`
  - Skill/plugin injection：`codex-rs/core/src/session/turn.rs`
  - 工具注册：`codex-rs/core/src/tools/spec_plan.rs`
  - TUI slash commands：`codex-rs/tui/src/slash_command.rs`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns red/failing tests, fixtures, assertions, snapshots if needed, `.pipeline/` detection, Workflow context injection, `/hw:*` routing, Question/Plan availability, and automatic Subagent guidance.
  - Expected artifact: `.pipeline/reviews/C6/M3/test.md`
- `implement`
  - Scope: owns runtime detector, context injection/routing implementation, and slash/skill integration.
  - Must not create, edit, or rewrite test assets.
  - Expected artifact: `.pipeline/reviews/C6/M3/implement.md`
- `audit`
  - Scope: checks non-Workflow compatibility, activation guard, permission gates, prompt bloat, and Worker Separation evidence.
  - Must reject fake tests that only inspect static strings without starting/building a turn context or routing a command.
  - Expected artifact: `.pipeline/reviews/C6/M3/audit.md`

Lifecycle evidence required before completion: requested, started, completed/failed/blocked, and closed/close_failed for all roles.

Destructive operations and system-level dependency installation require user confirmation.

## Technical Solution

Add Workflow-aware runtime routing activated by `.pipeline/` or explicit config. The runtime should inject concise Workflow capabilities and habits into model-visible context, prefer `request_user_input` for hard gates, prefer `update_plan` for visible planning discipline, and make `/hw:*` / `$hypo-workflow:*` feel native.

Automatic Subagent delegation should be encouraged in all suitable contexts. It must still stop for destructive operations and system-level dependency installation.

## Technical Route

1. Add Workflow project detection from cwd/project root.
2. Add a concise context contributor or equivalent section in `build_initial_context`; avoid bloating base instructions.
3. Add tests proving non-Workflow projects do not receive Workflow-specific context.
4. Inspect existing Skill injection behavior and implement context-aware Hypo-Workflow selection without breaking explicit skill mentions.
5. Add or route `/hw:*` slash command handling in TUI/app-server boundary using existing command patterns.
6. Ensure `request_user_input` and `update_plan` remain available and described for Workflow gate/plan use.
7. Update Subagent usage hints for Workflow planning/execution/explore contexts.
8. Write `.pipeline/reports/003-native-workflow-runtime-routing.md`.

## Research Required

Implementation must inspect the exact least-invasive layer for Skill auto-selection and slash dispatch. Do not hardwire behavior in a way that bypasses existing plugin/skill systems.

## Risks And Alternatives

- Risk: every project gets Workflow behavior. Guard on `.pipeline/` or config.
- Risk: prompt surface grows too much. Keep instructions compact and factual.
- Risk: automatic Subagent guidance conflicts with permission policy. Preserve confirmation gates for destructive/system-install operations.
- Rejected alternative: only edit `AGENTS.md`; user asked for internal default ability.

## Validation Path

Real tests must include:

- `.pipeline/` project gets Workflow-aware context.
- non-Workflow project does not.
- `/hw:plan` or equivalent routing is recognized.
- Request Questions/update_plan guidance is present in Workflow mode.
- Subagent guidance does not remove destructive/system-install gates.

Suggested commands:

- `just test -p codex-core workflow`
- `just test -p codex-tui slash`
- `just test -p codex-core spec_plan`
- `git diff --check`

Pass signal: Workflow behavior is native in `.pipeline/` projects and invisible or inert elsewhere.

## Audit Focus

- Activation guard.
- Prompt size.
- No permission bypass.
- Compatibility with explicit Skill mentions and existing slash commands.
