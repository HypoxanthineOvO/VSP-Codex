# C3 Ask User Tools 与 Plan 常驻窗口 Progress

更新时间：2026-05-27T16:35:00+08:00

## 当前状态

- 当前 Cycle：C3 `Ask User Tools 与 Plan 常驻窗口`
- 类型：feature
- Preset：tdd
- 阶段：completed
- 已完成：M1-M13；用户已明确要求 accept 并 close，M7 真实跨设备 pending Ask resume gate 按本次验收请求关闭
- 验收状态：accepted
- 修订输入：`.pipeline/acceptance/cycle-C3-rejection-20260526T214830+0800.yaml`
- 核心问题：重启后的普通 Default/Bypass 会话中，助手需要提问时必须实际使用 Ask Tools，并且 TUI 必须弹出原生 Ask UI；已补 core/tool 层修复、model-visible tools 测试、TUI native Ask capability 声明、remote initialize capability 断言，并重建本地 `vsp-codex` 入口

## 已确认决策

- Ask 基于现有 `request_user_input`，不新增平行工具族。
- Ask 可用于默认 bypass/ordinary turn，不限于 Plan mode。
- Subagent 不直接弹 Ask；由 root session 汇总询问。
- Ask 支持 request-level summary、question description、single-choice、multi-choice、free-text/short-answer、order-only ranking。
- app-server Ask API 在 C3 保持 experimental。
- native Ask UI capability 独立于 `experimentalApi`。
- Pending Ask 必须支持跨设备 resume；unsupported client 降级普通编号聊天问题，不能卡死。
- Secret/private 可见记录使用圆点掩码。
- Plan/SubPlan 改造现有 `update_plan`，保留 legacy flat checklist 兼容。
- SubPlan 首版只支持一层。
- TUI Plan panel 常驻，Default idle 折叠，运行中展开，支持 `Alt+P`。

## Milestone 结果

1. M1 Ask 协议模型与 app-server experimental schema：completed
2. M2 Core Ask 工具语义与模型契约：completed
3. M3 TUI Ask 多题型与隐私展示：completed
4. M4 Pending Ask resume 与 unsupported-client fallback：completed
5. M5 Plan/SubPlan structured state 与 app-server v2 暴露：completed
6. M6 TUI 常驻 Plan/SubPlan 面板：completed
7. M7 集成隐私回归与真实跨设备验收：accepted_for_close
8. M8 request_user_input 普通会话可用性修复：completed
9. M9 TUI native Ask UI capability 修复：completed
10. M10 Plan 强制更新门禁：completed
11. M11 Ask UI 卡片式体验重构与本地化：completed
12. M12 Subagent 与 Plan UI 一致性修复：completed
13. M13 Subagent 昵称候选池配置化：completed

## 拒绝反馈

用户拒绝 C3，因为核心要求是 Ask Tools 在普通 Default/Bypass 语境中也能使用，而实际重启当前会话后，助手在需要问两个选择问题时没有使用 Ask Tools。这说明 C3 不能只通过内部 schema/unit tests 验收，必须补充真实端到端交互验证。

## 主要代码产物

- `codex-rs/protocol/src/request_user_input.rs`
- `codex-rs/core/src/tools/handlers/request_user_input_spec.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
- `codex-rs/app-server/src/request_user_input.rs`
- `codex-rs/app-server/src/request_user_input_fallback.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
- `codex-rs/tui/src/history_cell/request_user_input.rs`
- `codex-rs/protocol/src/plan_tool.rs`
- `codex-rs/core/src/tools/handlers/plan.rs`
- `codex-rs/core/src/tools/handlers/plan_spec.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/turn.rs`
- `codex-rs/tui/src/chatwidget/plan_panel.rs`
- `codex-rs/tui/src/chatwidget/rendering.rs`
- `codex-rs/tui/src/chatwidget/interaction.rs`

## 验证记录

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

结果：上述命令通过；TUI snapshot 无 pending。

完整 workspace `cd codex-rs && just test` 未运行，因为 AGENTS.md 要求 common/core/protocol 改动后运行完整 suite 需先征得用户同意。

## 报告

- `.pipeline/reports/001-ask-protocol-app-server-schema.md`
- `.pipeline/reports/002-core-ask-tool-semantics.md`
- `.pipeline/reports/003-tui-ask-multikind-privacy.md`
- `.pipeline/reports/004-pending-ask-resume-fallback.md`
- `.pipeline/reports/005-plan-subplan-state-app-server.md`
- `.pipeline/reports/006-tui-persistent-plan-panel.md`
- `.pipeline/reports/007-integration-privacy-cross-device-acceptance.md`
- `.pipeline/reports/008-request-user-input-default-runtime-fix.md`
- `.pipeline/reports/010-plan-update-gate.md`
- `.pipeline/reports/011-ask-ui-card-experience.md`
- `.pipeline/reports/012-subagent-plan-ui-consistency.md`
- `.pipeline/reports/013-subagent-nickname-candidates-config.md`

## 追加计划

M12 已完成：

- Prompt：`.pipeline/prompts/012-subagent-plan-ui-consistency.md`
- 目标：统一 Subagent spawn 历史提示、active Subagent 列表和详情弹窗的昵称+角色显示；修复 Plan 折叠态在完成后仍显示旧步骤的问题。
- 执行策略：strict worker separation，test/implement/audit 分离完成。
- 报告：`.pipeline/reports/012-subagent-plan-ui-consistency.md`
- 结果：active Subagent compact/detail 使用 `<nickname> [<role>]`；Plan all-completed collapsed state 显示完成摘要。
- 验证：focused TUI tests、snapshot hygiene、fmt、fix、`cargo build --bin vsp-codex` 均通过；audit verdict 为 Pass。

## 下一步

已完成修订：

1. `request_user_input_available_modes()` 改为所有 collaboration modes。
2. `ModeKind::allows_request_user_input()` 改为所有 collaboration modes。
3. 工具描述改为 every collaboration mode。
4. 新增普通 Default turn 的 model-visible tools 断言。
5. Default/Execute/Pair request_user_input round-trip focused tests 通过。

真实复验结果：

- 第一次重启后，助手实际调用 Ask 工具，运行时返回 `request_user_input is unavailable in Default mode`。复查发现当前 `vsp-codex resume` 运行的是 M8 修复前的旧 debug 二进制。
- 第二次重启后，`request_user_input` 已能在 Default 模式运行，但 TUI 显示 unsupported-client fallback 文本，说明 TUI app-server client 未声明 `native_ask_ui=true`。

已执行 `cd codex-rs && cargo build --bin vsp-codex`，新的 `/home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex` 更新时间为 2026-05-26T22:51:47+08:00。

M9 修复：

1. `InProcessClientStartArgs` 和 `RemoteAppServerConnectArgs` 增加 `native_ask_ui`。
2. TUI embedded/remote app-server client 传 `native_ask_ui=true`。
3. exec、debug/test/onboarding 非 Ask UI 客户端保持 `false`。
4. 新增 app-server-client 初始化参数断言。
5. 新增 remote initialize payload 断言，直接验证 `nativeAskUi=true` 会出现在 remote app-server 初始化请求中。
6. 更新 app-server turn_start 测试断言，匹配 `request_user_input` 已扩展到 every collaboration mode 的新语义。

验证：

- `cd codex-rs && just fmt`
- `cd codex-rs && just test -p codex-app-server-client in_process_initialize_params_preserve_native_ask_ui_capability`
- `cd codex-rs && just test -p codex-app-server-client remote_server_request`
- `cd codex-rs && just test -p codex-app-server-client remote_initialize_preserves_native_ask_ui_capability`
- `cd codex-rs && just test -p codex-app-server request_user_input`
- `cd codex-rs && just test -p codex-app-server turn_start_uses_thread_feature_overrides_for_request_user_input_tool_description_v2`
- `cd codex-rs && just test -p codex-tui request_user_input`
- `cd codex-rs && just fix -p codex-app-server-client -p codex-app-server -p codex-tui -p codex-exec`
- `cd codex-rs && cargo build --bin vsp-codex`

结果：

- app-server-client in-process capability test：1 passed。
- app-server-client remote server request tests：2 passed。
- app-server-client remote initialize native Ask capability test：1 passed。
- app-server request_user_input tests：5 passed。
- app-server turn_start request_user_input description override test：1 passed。
- codex-tui request_user_input tests：78 passed，2596 skipped。
- `just fix -p codex-app-server-client -p codex-app-server -p codex-tui -p codex-exec`：通过。
- `cargo build --bin vsp-codex`：通过。

新的 `/home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex` 更新时间为 2026-05-26T23:38:44+08:00。

真实重启复验：

- 2026-05-26T23:48:12+08:00，用户重启 `vsp-codex resume` 后，助手在普通 Default 会话调用 `request_user_input`。
- TUI 成功显示原生 Ask UI；用户在 Ask UI 中选择 `native_ask_ui`。
- 用户同时反馈 Plan 在压缩或对话过长后可能被忘记更新，并认为 Ask UI/交互逻辑仍需要微调。

P3 追加规划：

- 已生成 `.pipeline/prompts/010-plan-update-gate.md`。
- 已生成 `.pipeline/prompts/011-ask-ui-card-experience.md`。
- `pipeline.prompts_total` 已扩展到 11。

M10 完成：

1. 新增 core Plan freshness 状态，跟踪 active Plan、stale 原因、一次性提醒和工具调用计数。
2. resume/fork 从 rollout 的 `PlanUpdate` 事件或 `update_plan` function call 参数恢复 active Plan，并标记为 stale。
3. compact 后标记 active Plan stale。
4. 连续 3 个非 `update_plan` 工具调用后标记 active Plan stale。
5. stale 后在模型上下文注入 `<plan_freshness_reminder>`，明确要求继续实质性工作前调用 `update_plan`。
6. 新的 `update_plan` 会清除 stale gate，fresh Plan 不重复提醒。

M10 验证：

- `cd codex-rs && just fmt`
- `cd codex-rs && just test -p codex-core record_context_updates_injects_plan_freshness_reminder_once_when_stale fresh_plan_update_clears_plan_freshness_reminder tool_burst_marks_active_plan_stale_until_update_plan`
- `cd codex-rs && just test -p codex-core plan_update_gate`
- `cd codex-rs && just fix -p codex-core`

结果：focused session tests 3 passed；`plan_update_gate` integration tests 3 passed；scoped fix passed。按仓库规则，`just fix` 后未重复跑测试。

M11 完成：

1. zh-CN Ask UI 增加中文进度、未答数量、Other、讨论入口、键盘提示和未答确认文案。
2. zh-CN option 渲染改为边框卡片，选项标题和描述分层展示。
3. 未答确认列出未答问题，默认选中 `返回修改`，Enter 行为与视觉顺序一致。
4. 新增并接受 zh-CN 卡片层级和未答确认 snapshot。
5. audit worker 只读审计通过；残余风险记录为 CJK 启发式本地化、Other legacy 内部英文 label、仍需真实 TUI 体验验收。

M11 验证：

- `cd codex-rs && just test -p codex-tui request_user_input_zh_card_hierarchy_tabs_snapshot request_user_input_zh_unanswered_confirmation_snapshot`：2 passed。
- `cd codex-rs && just test -p codex-tui request_user_input`：80 passed。
- `cd codex-rs && cargo insta pending-snapshots --workspace`：No pending snapshots。
- `cd codex-rs && just fmt`：通过。
- `cd codex-rs && just fix -p codex-tui`：通过。

下一步：仍需完成 M7 的真实换电脑/跨设备 pending Ask resume 验收，C3 才能最终验收。

## M13 完成

M13 已完成：

- Prompt：`.pipeline/prompts/013-subagent-nickname-candidates-config.md`
- 目标：把 Subagent nickname candidates 配置化，支持项目/全局配置、中文 Unicode 名字、core spawn 与 TUI fallback 一致。
- 默认池：公共默认使用中国古代人物名，避免发布后使用私人姓名。
- 个人池：通过配置支持当前自用中文名单，并追加 `陈阳宾`、`尹一帆`，不写死进公共默认。
- 报告：`.pipeline/reports/013-subagent-nickname-candidates-config.md`
- 结果：新增顶层 `[agents].nickname_candidates`，保留 role-level override；core spawn 与 TUI active fallback 都按 role-level configured > top-level configured > built-in role/default 的优先级选择昵称；schema 已暴露新字段；公共默认池已改为中国古代人物名。
- 验证：`just write-config-schema`、focused core tests、focused TUI fallback tests、snapshot hygiene、fmt、scoped fix、`cargo build --bin vsp-codex` 均通过。
- 审计：首轮 audit 发现 TUI fallback role-specific gap；修复后复审 verdict 为 pass。

## 验收关闭

用户在 2026-05-27 明确要求 “先 Accept 然后 Close”，因此 C3 已按手动验收关闭。此前 M7 的真实跨设备 pending Ask resume gate 记录为 accepted_for_close。
