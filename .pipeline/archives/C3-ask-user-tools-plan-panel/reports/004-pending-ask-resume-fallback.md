# M4 Report: Pending Ask resume 与 unsupported-client fallback

完成时间：2026-05-26T21:32:13+08:00

## 结果

已完成本地机制。Pending Ask 的 durable metadata、native Ask capability 分支和 unsupported-client fallback 已实现；真实跨设备 resume 仍保留为 M7 用户手动验收门。

## 主要变更

- `codex-rs/app-server/src/request_user_input.rs`
  - 集中处理 core/app-server Ask 参数与 response 转换。
- `codex-rs/app-server/src/request_user_input_fallback.rs`
  - 构造 unsupported client 的普通编号聊天 fallback。
- `codex-rs/app-server/src/thread_state.rs`
  - 保存 pending unsupported Ask 映射。
- `codex-rs/app-server/src/request_processors/thread_lifecycle.rs`
  - resume/read 路径可重放 pending Ask 或 fallback。
- `codex-rs/rollout/src/policy.rs`
  - 保留 `RequestUserInput` 事件用于有限历史恢复。

## 验证

- `cd codex-rs && just test -p codex-app-server request_user_input test_handle_turn_plan_update_emits_notification_for_v2`
- `cd codex-rs && just test -p codex-tui plan_panel request_user_input`

结果：通过。

## 风险

跨进程/跨设备场景中，原 core suspended callback 无法在新进程中继续时，会改为重新展示 pending Ask 或 fallback，并通过新的普通用户 turn 上下文继续。这避免卡死，但需要 M7 真实跨设备手动验收确认产品体验。
