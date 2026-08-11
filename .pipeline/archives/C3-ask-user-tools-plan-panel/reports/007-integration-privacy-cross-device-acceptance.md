# M7 Report: 集成隐私回归与真实跨设备验收

完成时间：2026-05-26T21:32:13+08:00

## 结果

自动验证部分已完成；C3 最终验收阻塞在用户真实跨设备 pending Ask resume 测试。按 Deep Plan 决策，在该手动门槛通过前不能执行 `/hw:accept`。

## 自动验证证据

- `cd codex-rs && just fmt`
- `cd codex-rs && just test -p codex-core update_plan`
- `cd codex-rs && just test -p codex-app-server-protocol turn_plan_updated_round_trip_structured_state`
- `cd codex-rs && just write-app-server-schema --experimental`
- `cd codex-rs && cargo insta pending-snapshots --manifest-path tui/Cargo.toml`
- `cd codex-rs && just test -p codex-tui plan_panel plan_update_renders_history_cell plan_panel_collapses_when_idle_and_expands_while_running alt_p_toggles_plan_panel_expansion`
- `cd codex-rs && just test -p codex-tui plan_panel request_user_input`
- `cd codex-rs && just test -p codex-app-server request_user_input test_handle_turn_plan_update_emits_notification_for_v2`
- `cd codex-rs && just write-app-server-schema`
- `cd codex-rs && just test -p codex-app-server-protocol generated_ts_optional_nullable_fields_only_in_params`
- `cd codex-rs && just test -p codex-app-server-protocol`
- `cd codex-rs && just fmt && just fix -p codex-app-server -p codex-tui`

结果：全部通过；snapshot 无 pending。

## 未运行项

完整 workspace `cd codex-rs && just test` 未运行，因为本仓库 AGENTS.md 要求 common/core/protocol 改动后运行完整 suite 需先征得用户同意。

## 手动验收门

需要用户完成：

1. 在一台电脑启动一个会触发 pending Ask 的会话。
2. 不回答 Ask，换另一台电脑 resume 同一会话。
3. 支持 native Ask UI 的客户端应重新弹出 Ask。
4. 不支持 native Ask UI 的客户端应显示普通编号聊天 fallback，不能卡死。
5. 如果任一行为失败，C3 继续保持 active，并通过 `/hw:plan:extend` 添加修复 Milestone。

## 状态

`M1-M6 completed`，`M7 blocked: manual_cross_device_pending_ask_resume`。
