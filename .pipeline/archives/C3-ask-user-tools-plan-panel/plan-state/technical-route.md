# C3 P2 Technical Route：Ask User Tools 与 Plan 常驻窗口

状态：`proposed`  
时间：2026-05-26T18:09:31+08:00  
来源：[p1-discover.yaml](p1-discover.yaml)、[plan-context.md](../deep-plans/DP001-ask-user-tools/plan-context.md)

本阶段只提出 Milestone 与技术路线，不生成执行 prompt。进入 P3 Generate 前需要用户确认。

## 总体顺序

1. M1 Ask 协议模型与 app-server experimental schema
2. M2 Core Ask 工具语义与模型契约
3. M3 TUI Ask 多题型与隐私展示
4. M4 Pending Ask resume 与 unsupported-client fallback
5. M5 Plan/SubPlan structured state 与 app-server v2 暴露
6. M6 TUI 常驻 Plan/SubPlan 面板
7. M7 集成隐私回归与真实跨设备验收

顺序理由：先固定跨客户端 wire contract，再改 core 工具语义和 TUI 原生体验；随后处理恢复和 fallback。Plan/SubPlan 先建立 canonical structured state 和 app-server surface，再做 TUI 常驻面板。最后统一做隐私、schema、snapshot、focused tests 和真实跨设备验收。

## M1 Ask 协议模型与 app-server experimental schema

目标：把 `request_user_input` 扩展为可表达题型、总说明、逐题说明、结构化答案、secret 掩码和 native Ask capability 的跨客户端 experimental 契约。

技术方案：
- 扩展 `codex_protocol::request_user_input`，增加 request-level summary、question kind、per-question description、option id、结构化 answer。
- 扩展 app-server v2 `ToolRequestUserInput*` 类型，继续作为 experimental surface。
- native Ask UI 支持使用独立 client capability，不复用 `experimentalApi`。
- 保留旧 `answers: Vec<String>` 或兼容转换，避免现有路径一次性断裂。

触达模块：
- `codex-rs/protocol/src/request_user_input.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
- `codex-rs/app-server-protocol/src/protocol/v1.rs`
- `codex-rs/app-server/src/bespoke_event_handling.rs`
- app-server schema fixtures

验证：
- `just write-app-server-schema`
- `just write-app-server-schema --experimental`
- `just test -p codex-app-server-protocol`

审计重点：
- Ask API 仍 experimental，未误稳定化。
- native Ask capability 与 `experimentalApi` 分离。
- option id、order、freeText、secret 语义不丢失。

## M2 Core Ask 工具语义与模型契约

目标：让 root agent 在普通交互 turn/default bypass 中使用 Ask，并支持“问题 + 简述 + 回答模块”、多题型 normalization、secret 策略和模型防滥用说明。

技术方案：
- 更新 `request_user_input` tool schema，支持 request summary、question kind、per-question description、single/multi/short/free/ranking。
- 按题型校验参数，不再强制每题 options 非空。
- 不做 per-turn 或每次题数硬限制；通过工具说明约束“合适时才问”。
- 保持 root-thread-only；Subagent 不能直接向用户弹 Ask。
- UI-less/unattended 场景必须 fallback 或明确失败。

触达模块：
- `codex-rs/core/src/tools/handlers/request_user_input_spec.rs`
- `codex-rs/core/src/tools/handlers/request_user_input.rs`
- `codex-rs/core/src/session/mod.rs`
- `codex-rs/core/src/session/tests.rs`

验证：
- `just test -p codex-core request_user_input`
- `just test -p codex-core pending_request_user_input_does_not_spawn_extra_goal_continuation`

审计重点：
- 没有加入用户不想要的硬限制。
- 默认 bypass/ordinary interactive 可用。
- Subagent 仍不能直接弹 Ask。

## M3 TUI Ask 多题型与隐私展示

目标：扩展现有 bottom pane overlay，支持 multi choice、short/free text、order-only ranking、request/question 说明和 secret 圆点掩码。

技术方案：
- 在 `RequestUserInputOverlay` 中引入题型化 answer state。
- single 复用现有 selection；multi 使用 selected set；ranking 使用 ordered option ids；free/short 使用 composer draft。
- render 层显示 request summary 和 per-question description。
- history/result/raw lines 对 secret 使用圆点掩码。

触达模块：
- `codex-rs/tui/src/bottom_pane/request_user_input/`
- `codex-rs/tui/src/history_cell/request_user_input.rs`
- `codex-rs/tui/src/app/app_server_requests.rs`

验证：
- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`

审计重点：
- 不扩展 `chatwidget.rs` 承担 overlay 逻辑。
- ranking 是 order-only，不加入 per-item notes。
- visible/raw history 都不泄露 secret。

## M4 Pending Ask resume 与 unsupported-client fallback

目标：pending Ask 跨设备 resume 能恢复 native UI；不支持 native Ask UI 时降级为普通编号聊天问题，不能卡死。

技术方案：
- 扩展 pending interactive replay 中的 RequestUserInput metadata，不只保存 turn/item id。
- 使用 native Ask capability 决定 replay native request 还是生成 fallback chat prompt。
- fallback prompt 使用编号、题型、选项 id/label 和回答格式示例。
- 解析失败时普通对话追问澄清；原 turn 不可恢复时显式降级为新 turn 上下文。

触达模块：
- `codex-rs/tui/src/app/pending_interactive_replay.rs`
- `codex-rs/tui/src/app/app_server_requests.rs`
- `codex-rs/app-server/src/thread_state.rs`
- `codex-rs/app-server/src/bespoke_event_handling.rs`

验证：
- `just test -p codex-tui pending_interactive_replay`
- `just test -p codex-app-server`
- `just test -p codex-core request_user_input`

审计重点：
- 没有 deadlock path。
- unsupported client 不等待 native UI。
- resolved Ask 不会重复弹出。

## M5 Plan/SubPlan structured state 与 app-server v2 暴露

目标：把 `update_plan` 演进为兼容 flat checklist 的结构化 Plan/SubPlan 工具，并通过 app-server v2 暴露 canonical state。

技术方案：
- 扩展 `plan_tool.rs`，新增 PlanState、root item、stable id、summary、一层 SubPlan。
- `update_plan` 保留 legacy flat checklist 输入，并归一化为 structured state。
- structured state 是常驻面板 source of truth；proposed-plan markdown 只做 proposal/history。
- app-server v2 暴露 structured Plan/SubPlan state 给 TUI 和未来 VSCode/GUI。

触达模块：
- `codex-rs/protocol/src/plan_tool.rs`
- `codex-rs/core/src/tools/handlers/plan_spec.rs`
- `codex-rs/core/src/tools/handlers/plan.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/turn.rs`
- `codex-rs/app-server/src/bespoke_event_handling.rs`

验证：
- `just write-app-server-schema --experimental`
- `just test -p codex-app-server-protocol`
- `just test -p codex-core update_plan`

审计重点：
- 没有新增割裂的平行计划工具。
- SubPlan 只做一层。
- future clients 不需要解析 TUI history cell。

## M6 TUI 常驻 Plan/SubPlan 面板

目标：在 TUI 输入框上方右侧常驻显示 structured Plan/SubPlan state，Default idle 折叠为一行摘要并支持快捷键展开。

技术方案：
- 新增独立 Plan panel 模块，消费 M5 structured PlanState。
- 宽屏显示 root plan 和 active SubPlan；窄屏/idle 折叠为 root item、SubPlan、completed/total。
- 提供快捷键展开折叠态详情。
- panel 只读，不支持手动编辑。

触达模块：
- 新增 `codex-rs/tui/src/.../plan_panel` 或 bottom pane 邻近模块
- `codex-rs/tui/src/chatwidget/protocol.rs`
- `codex-rs/tui/src/bottom_pane/mod.rs`
- TUI keymap/help hints

验证：
- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`

审计重点：
- Default idle 折叠仍能快捷键展开。
- Ask overlay 与 Plan panel 不重叠。
- panel 不从 markdown/history cell 解析状态。

## M7 集成隐私回归与真实跨设备验收

目标：把 Ask、fallback、Plan/SubPlan、TUI panel 和 app-server state 串成可交付闭环，并完成最终隐私与跨设备验收。

技术方案：
- 统一跑 schema、focused tests、snapshot、fmt、fix。
- 审查 secret/private answer 在 history/transcript/logs/summaries 中的掩码。
- 本地 smoke 覆盖 Ask UI、fallback、Plan panel、PlanState notification。
- 最终等待用户真实换电脑 resume pending Ask。失败则本 Cycle extend milestone 回修。

验证：
- `just fmt`
- `just write-app-server-schema --experimental`
- `just test -p codex-app-server-protocol`
- `just test -p codex-core`
- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`

人工验收：
- 用户在一台电脑发起 pending Ask。
- 换另一台电脑 resume。
- 支持 native Ask UI 的客户端重新弹同一个 Ask。
- 不支持 native Ask UI 的客户端走普通编号聊天 fallback。

审计重点：
- 自动测试不能替代真实跨设备验收。
- secret/private 不泄露。
- 如果真实 resume 失败，C3 不可接受，只能 extend 回修。

## 未解决假设

- native Ask capability 字段名留给实现阶段按 repo 风格命名。
- Plan panel responsive breakpoint 在 M6 实现阶段根据 TUI 布局测试确定。
- fallback 回复解析失败时优先使用普通对话追问澄清；具体 parser/core handoff 由 M4 实现阶段定型。
