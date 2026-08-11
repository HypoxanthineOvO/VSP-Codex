# Milestone 2: TUI 用户模拟测试框架补强

## Execution Boundary

本提示只执行 Milestone 2。目标是在已转绿的 `codex-tui` 基线上补强 TUI 用户模拟测试框架。

不要开始以下实现：

- `vsp-codex` 二进制/入口改名。
- `~/.vsp-codex` home 目录切换或首次导入 UI。
- `/provider` 功能实现。
- 官方 Cloud/OAuth/ChatGPT app/update/feedback 功能隐藏或禁用。
- VSP 品牌、中文文案或 UI 美化。

允许改动范围仅限测试框架、测试 helper、测试 snapshot，以及本 Milestone 必需的 workflow 记录。

## Subworker Assignment Plan

Worker Separation: `strict`。用户已授权 Codex 在 `/hw:start` 和 `/hw:resume` 中使用执行 Subagent，角色必须分离。

### test

- Role status: authorized.
- Ownership: 设计并实现 M2 的 TUI 用户模拟测试框架和基线测试。
- Allowed write paths:
  - `codex-rs/tui/src/chatwidget/tests.rs`
  - `codex-rs/tui/src/chatwidget/tests/helpers.rs`
  - `codex-rs/tui/src/chatwidget/tests/user_simulation.rs`
  - `codex-rs/tui/src/chatwidget/snapshots/*.snap`
  - `.pipeline/reviews/002-tui-user-simulation-test.md`
- Must not edit production behavior, manifests, lockfiles, build scripts, or non-test Rust modules.
- Expected evidence:
  - 新增或整理可复用的 TUI user-flow harness，尽量通过 `KeyEvent` 或现有 chatwidget public/test helpers 模拟真实用户输入。
  - 至少覆盖 slash popup、选择/弹窗渲染、普通 prompt 提交链路中的两个以上稳定场景。
  - 生成的 snapshot 必须可审查且无意外 `.snap.new` 残留。

### implement

- Role status: authorized.
- Ownership: 本 Milestone 默认无生产实现。仅检查是否需要生产代码改动并写 no-op implementation review。
- Allowed write paths:
  - `.pipeline/reviews/002-tui-user-simulation-implement.md`
- Must not edit `codex-rs/` 源码、测试、snapshots、manifest、lockfile 或 build logic。
- Expected evidence: 确认 M2 是测试框架 Milestone，无需生产实现。

### audit

- Role status: authorized.
- Ownership: 只读审查最终 diff、测试证据、snapshot 范围、worker separation 和后续 M3-M6 可用性。
- Allowed write paths:
  - `.pipeline/reviews/002-tui-user-simulation-audit.md`
- Must not edit source or测试报告主体。

### main agent

- 编排 worker 生命周期，集成 test worker 的结果，运行验证命令，更新 lifecycle 状态。
- 不得本地承担 test、implement、audit 任一角色。

## Technical Solution

在现有 `codex-rs/tui` 测试体系上补强可复用的 TUI 用户流程测试能力。优先复用 `make_chatwidget_manual`、`render_bottom_popup`、chatwidget snapshots、现有 `KeyEvent` 驱动和 tmux ignored smoke，不引入新 UI 测试框架。

M2 应提供一个后续 M3-M6 能复用的轻量测试层，用于验证：

- 用户键入 slash 命令时的 popup/filter 行为。
- 用户选择一个 popup 命令后，进入对应选择/弹窗界面。
- 用户提交普通 prompt 时，真实提交链路产生 `Op::UserTurn`。

## Technical Route

1. 基于现有 `chatwidget` 测试 helper 新增一个 focused user-flow harness。
2. 通过 key events 或现有 chatwidget test API 模拟用户键入、Enter/Esc/Tab 等操作。
3. 新增稳定 baseline 测试，不引入 VSP 功能断言：
   - slash command popup/filter baseline。
   - model picker 或现有选择弹窗 baseline。
   - 普通 prompt 提交流程 baseline。
4. 对用户可见 popup 使用 snapshot 或明确字符串断言。
5. 保持变更小，不重构 `chatwidget.rs`、`bottom_pane` 大模块或生产 UI。

## Research Required

M1 已完成并修复当前本机基线：

- `just test -p codex-tui` 当前通过。
- `just fmt` 当前通过。
- tmux ignored resize/reflow smoke 当前通过。

仍需在 M2 内确认新增 harness 的稳定测试命令和 snapshot hygiene。

## Risks And Alternatives

- 风险：过多 snapshot 会造成 churn。替代方案是只为关键 popup 加 snapshot，其余用结构化断言。
- 风险：测试 helper 过度暴露内部实现。替代方案是把 helper 设计成“用户动作”语义：type/press/open/submit/render，而不是直接改内部状态。
- 风险：M2 无意中开始实现 `/provider` 或 VSP UI。必须拒绝；这些属于 M3-M6。
- 拒绝：为了测试方便重构 TUI 生产模块。

## Validation Path

必须运行：

```bash
just fmt
just test -p codex-tui chatwidget::tests::user_simulation
just test -p codex-tui
cargo insta pending-snapshots
git diff --check
```

如果可用且耗时可接受，运行：

```bash
just test -p codex-tui resize_reflow -- --ignored
```

不要在未再次获得用户确认前运行完整 workspace `just test`。

## Audit Focus

- 新增测试是否模拟真实用户可观察行为，而不是只测私有函数。
- 是否没有开始任何 VSP 功能实现。
- 是否遵守 snapshot 要求，且没有留下 `.snap.new`。
- 是否为后续 M3-M6 提供可复用测试入口。
- 是否遵守 Worker Separation。

## Completion Criteria

- M2 测试框架和基线测试已合入当前工作树。
- `.pipeline/reviews/002-tui-user-simulation-test.md`、`002-tui-user-simulation-implement.md`、`002-tui-user-simulation-audit.md` 已生成。
- `just fmt`、targeted user_simulation 测试、`just test -p codex-tui` 通过。
- 无意外 pending snapshots 或 `.snap.new`。
- 当前分支可提交并推送到 GitLab，作为后续 M3-M6 的测试框架版本。
