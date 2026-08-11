# M3 Subagent 常驻卡片与键盘查看

## Goal
在 TUI 中增加更突出的 Subagent 展示区域。运行中的 Subagent 需要在主对话下方以独立卡片/列表展示，格式接近：`杨超 · GPT 5.5 xhigh · Audit · 运行中 · 24h`。已关闭的 Subagent 不常驻显示。用户应能通过键盘进入查看 Subagent 的命令、返回内容和状态详情。

## Subworker Assignment Plan
Subworkers are authorized for this milestone. Worker separation is strict.

- `test`: 先写 TUI snapshot 和键盘交互测试，覆盖无 Subagent、单个运行中、多个运行中、完成后消失、详情面板打开/关闭、窄屏截断。测试 worker 只改测试与 snapshots。
- `implement`: 新增或扩展 TUI Subagent panel 模块，接入 M2 状态元数据，实现键盘导航和详情查看。实现 worker 不改核心 policy。
- `audit`: 审核 UI 状态生命周期、键盘冲突、滚动/焦点行为、中文宽度和 snapshot 稳定性。
- Main agent: 协调严格分工，处理跨模块接线，运行 TUI tests。

## Technical Solution
在 TUI 对话流和 composer 之间增加 Subagent panel。Panel 只展示 active/running/waiting 状态；completed/closed 默认从常驻区移除，但详情历史可以由后续 transcript 或日志入口查看。Panel 需要使用 M2 暴露的 Subagent 状态元数据，不从聊天文本中解析。

交互设计：

- 默认展示紧凑列表，每行包含中文名、model、reasoning effort、role、状态、运行时间。
- 当前有 active Subagent 时 panel 常驻；没有 active Subagent 时不占空间。
- 键盘可聚焦 panel 并打开详情视图，查看命令、最近输出、等待/运行状态和 agent id。
- 原始英文 Subagent 返回内容是否翻译由 M4 处理，M3 只预留内容区域和数据通道。

## Technical Route
1. 确认 TUI 布局树：chat transcript、working area、composer、bottom status 的边界。
2. 增加 Subagent panel 的渲染测试和键盘状态测试。
3. 新建小模块承载 panel 渲染，避免继续膨胀 `chatwidget.rs` 或 central orchestration files。
4. 接入 active Subagent 状态 store，保证状态变化触发 redraw。
5. 增加详情视图/弹层/展开区，按现有 TUI 风格处理焦点和 `esc` 返回。
6. 更新 snapshots 并检查中文名称宽度。

## Research Required
不需要外部 research。需要本地代码探索：

- TUI 当前如何表示 `Working`、tool call、agent event。
- 是否已有 keyboard focus/tab stack/overlay 模式可以复用。
- M2 状态元数据是否已足够；不足时只做小范围补充。

## Risks And Alternatives
- 风险：在 transcript 中插入 panel 会影响滚动锚点。替代方案是把 panel 放在 composer 上方的固定区域。
- 风险：键盘快捷键与现有 TUI 冲突。替代方案是复用已有 panel navigation 模式或只在 panel 可见时启用局部按键。
- 风险：active/closed 生命周期边界不清晰。替代方案是以 runtime status 为准，只要 status 为 running/waiting/spawned 就显示。

## Validation Path
- `just fmt` from `codex-rs`.
- `just test -p codex-tui`.
- `cargo insta pending-snapshots -p codex-tui`, review `.snap.new`, then `cargo insta accept -p codex-tui` only for intended UI changes.
- Manual TUI smoke: create at least two Subagents, verify panel list, open details, close details, then verify completed Subagent leaves active panel.

## Audit Focus
- Panel is backed by structured state, not fragile text parsing.
- No nested card styling or layout overlap.
- Long Chinese names, model names, and durations truncate predictably.
- Closed Subagents do not permanently occupy the active panel.

## Deliverables
- Subagent panel component/module with tests and snapshots.
- Keyboard detail view for active Subagents.
- Milestone report with visual evidence or snapshot references.
