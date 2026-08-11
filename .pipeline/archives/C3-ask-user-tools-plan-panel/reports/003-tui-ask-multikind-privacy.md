# M3 Report: TUI Ask 多题型与隐私展示

完成时间：2026-05-26T21:32:13+08:00

## 结果

已完成。TUI 现有 `request_user_input` overlay 已支持单选、多选、自由文本、order-only ranking、request/question 说明和 secret 圆点掩码。

## 主要变更

- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
  - `AnswerState` 支持 multi-select set、ranking order 和 free-text draft。
  - 多选使用 Space toggle。
  - ranking 使用 `Ctrl+J` / `Ctrl+K` 调整顺序。
  - submit 生成结构化答案，同时保留 legacy labels。
  - summary、description 和 question description 进入 overlay 渲染。
- `codex-rs/tui/src/history_cell/request_user_input.rs`
  - secret/private 可见历史使用圆点掩码。
- 新增/更新 TUI snapshot。

## 验证

- `cd codex-rs && just test -p codex-tui plan_panel request_user_input`
- `cd codex-rs && cargo insta pending-snapshots --manifest-path tui/Cargo.toml`

结果：通过，无 pending snapshot。

## 风险

ranking 首版只保存顺序，不支持 per-item notes；这是 Deep Plan 已确认的 C3 范围。
