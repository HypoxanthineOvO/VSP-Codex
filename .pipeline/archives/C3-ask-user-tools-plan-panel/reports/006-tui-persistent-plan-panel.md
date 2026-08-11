# M6 Report: TUI 常驻 Plan/SubPlan 面板

完成时间：2026-05-26T21:32:13+08:00

## 结果

已完成。TUI 现在维护最新 structured Plan state，并在输入框上方渲染常驻 Plan/SubPlan 面板；Default idle 折叠，运行中或用户快捷键展开。

## 主要变更

- `codex-rs/tui/src/chatwidget/plan_panel.rs`
  - 新增 Plan panel renderable。
  - collapsed 模式显示当前 root/subplan 和完成进度。
  - expanded 模式显示 root items 与一层 SubPlan。
  - 新增 snapshot 覆盖。
- `codex-rs/tui/src/chatwidget/transcript.rs`
  - 保存 `latest_plan_state`。
- `codex-rs/tui/src/chatwidget/turn_runtime.rs`
  - plan update 时更新最新 plan state。
- `codex-rs/tui/src/chatwidget/rendering.rs`
  - 在 bottom composer 上方插入 plan panel。
- `codex-rs/tui/src/chatwidget/interaction.rs`
  - `Alt+P` 切换 panel 展开状态。
- `codex-rs/tui/src/chatwidget/tests/plan_mode.rs`
  - 覆盖 idle 折叠、运行中展开和快捷键。

## 验证

- `cd codex-rs && just test -p codex-tui plan_panel plan_update_renders_history_cell plan_panel_collapses_when_idle_and_expands_while_running alt_p_toggles_plan_panel_expansion`
- `cd codex-rs && just test -p codex-tui plan_panel request_user_input`
- `cd codex-rs && cargo insta pending-snapshots --manifest-path tui/Cargo.toml`

结果：通过，无 pending snapshot。

## 风险

面板为只读；用户手动编辑不属于 C3。
