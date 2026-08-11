---
archived_source: .pipeline/archives/C3-ask-user-tools-plan-panel
cycle: C3-ask-user-tools-plan-panel
finished: 2026-05-27T16:35:00+08:00
kind: cycle-summary
started: 2026-05-26T17:11:55+08:00
status: closed
---

# Ask User Tools 与 Plan 常驻窗口 总结

## 目的与边界

Delivered Ask User Tools, pending Ask resume/fallback, Plan/SubPlan structured state and persistent TUI panel, Plan freshness reminders, zh-CN card-style Ask UI, Subagent display consistency, and configurable Subagent nickname pools.

## 最终结果

旧 Cycle 状态：`completed`。本预览不重新判断旧结果，只建立可读导航。

## 验证结果

- 13 个历史 milestone 已映射。
- 60 个旧文件继续保存在 `.pipeline/archives/C3-ask-user-tools-plan-panel`。

## 重要决定与经验

- 旧 `cycle.yaml` 没有结构化 lessons；请查看原始 Summary。

## 后续候选

- 不自动继承旧任务；新 Cycle 应通过 `builds_on` 选择需要的结果或经验。

## 映射缺口

- 无结构性缺失。

## 旧总结原文

# Cycle C3：Ask User Tools 与 Plan 常驻窗口

- Cycle：C3
- 名称：Ask User Tools 与 Plan 常驻窗口
- 类型：feature
- 状态：completed / accepted
- 开始时间：2026-05-26T17:11:55+08:00
- 结束时间：2026-05-27T16:35:00+08:00
- Preset：tdd
- 归档路径：`.pipeline/archives/C3-ask-user-tools-plan-panel/`

## Milestones

| Milestone | 一句话结果 |
|---|---|
| M1 | 建立 Ask request/answer 协议模型与 app-server experimental schema。 |
| M2 | 实现 core `request_user_input` 工具语义、fallback 输出和模型契约。 |
| M3 | TUI 支持 Ask 多题型、隐私展示和基础交互。 |
| M4 | Pending Ask 支持 resume 和 unsupported-client fallback，避免换端卡死。 |
| M5 | `update_plan` 改造为 Plan/SubPlan structured state 并通过 app-server v2 暴露。 |
| M6 | TUI 增加常驻 Plan/SubPlan panel。 |
| M7 | 集成隐私回归与跨设备验收 gate；用户最终要求 accept/close，记录为 accepted_for_close。 |
| M8 | 修复普通 Default/Bypass 会话中 `request_user_input` 不可用的问题。 |
| M9 | 修复 TUI native Ask UI capability，真实重启后原生 Ask UI 可显示。 |
| M10 | 增加 Plan freshness gate，压缩/resume/tool burst 后提醒模型刷新 Plan。 |
| M11 | Ask UI 改为 zh-CN 卡片式体验，增强标题/问题/选项层级和未答确认。 |
| M12 | 统一 Subagent spawn/active list/detail 昵称+角色显示，并修复 Plan completed 折叠摘要。 |
| M13 | Subagent nickname candidates 配置化，支持中文配置和古代人物公共默认池。 |

## 关键数据

- `just write-app-server-schema` / `--experimental`：通过相关阶段。
- `just write-config-schema`：通过，`[agents].nickname_candidates` 已进入 schema。
- `just test -p codex-core ...request_user_input...`、`...update_plan...`、`...plan_update_gate...`、M13 nickname focused tests：通过。
- `just test -p codex-app-server-protocol` 与 focused app-server tests：通过。
- `just test -p codex-app-server request_user_input` 等 focused tests：通过。
- `just test -p codex-tui request_user_input`、Plan panel tests、Subagent fallback tests：通过。
- `cargo insta pending-snapshots --workspace`：No pending snapshots。
- `just fmt`：通过。
- scoped `just fix`：通过。
- `cargo build --bin vsp-codex`：通过。

## 改动摘要

C3 把模型主动询问用户的能力产品化为 Ask User Tools，并补齐普通会话、TUI 原生 Ask UI、unsupported fallback、pending resume 和跨端恢复语义。同时将 Plan 工具从一次性 checklist 升级为可常驻展示的 structured Plan/SubPlan 状态，解决长上下文/压缩后模型忘记更新 Plan 的问题。后续追加修复增强了 Ask UI 的中文卡片式体验、Subagent 昵称/角色一致性，以及 Subagent nickname candidate 的可配置能力。

## 技术思路

核心路线是协议先行、runtime 可恢复、TUI 原生体验兜底：Ask 使用工具调用承载结构化问题，客户端支持时显示原生 UI，不支持时降级为编号文本；pending Ask 保存在会话状态中，resume 时重新发起客户端展示。Plan 复用并扩展 `update_plan`，由模型负责 structured state，TUI 负责常驻 panel 展示和折叠策略。Subagent 命名则把用户私有池移出公共默认，改为配置驱动并保留 deterministic fallback。

## 修改文件 / 模块

- Protocol/core tools：`codex-rs/protocol/src/request_user_input.rs`、`codex-rs/protocol/src/plan_tool.rs`、`codex-rs/core/src/tools/handlers/*`。
- App-server：`codex-rs/app-server*` 与 `codex-rs/app-server-protocol/*`。
- TUI Ask/Plan/Subagent：`codex-rs/tui/src/bottom_pane/request_user_input/`、`codex-rs/tui/src/history_cell/`、`codex-rs/tui/src/chatwidget/`。
- Config/Subagent roles：`codex-rs/config/src/config_toml.rs`、`codex-rs/core/src/config/*`、`codex-rs/core/src/agent/*`。
- Workflow 证据：`.pipeline/prompts/`、`.pipeline/reports/`、`.pipeline/reviews/C3/`、`.pipeline/deep-plans/`。

## 测试设计

测试按协议、core 工具、app-server schema/round-trip、TUI rendering/snapshot、resume/pending 行为、Subagent fallback 和配置 schema 分层。UI 可见变更均通过 focused TUI tests 与 insta pending snapshot 检查；复杂修复采用 strict worker separation，保留 test/implement/audit 证据。

## 验证结果

目标路径均已通过 focused 验证和最终 `vsp-codex` build。完整 workspace `just test` 未运行，因为当前 Cycle 涉及 core/common/protocol/TUI/app-server 大范围改动，仓库指令要求完整 suite 需先征得用户同意；本轮按 focused regressions 和 build 进行收口。

## 预期结果

模型可在普通 Default/Bypass 会话中用原生 Ask UI 向用户提问；TUI/未来 GUI 可以消费结构化 Ask；unsupported client 不会卡死；Plan panel 常驻并能展示 SubPlan；压缩或长上下文后会提醒刷新 Plan；Subagent 昵称和角色显示一致，昵称池可由用户配置。

## 遇到的问题

- 初始实现错误地限制 `request_user_input` 到非 Default 场景，真实重启后暴露问题。
- TUI native Ask capability 未正确声明，导致原生 UI 退化为 unsupported fallback。
- Ask UI 初版层级、语言、Other/Chat About This 体验不足，需要 M11 重构。
- Plan panel 在完成后曾保留旧 focused step，需要 M12 修复。
- M13 schema 与 flatten role map 有冲突，需要额外 schema/deserialization 处理。

## 风险 / 后续

- M7 的真实跨设备 pending Ask resume 由用户本轮明确要求 accept/close，未保留为阻塞项；未来仍建议在真实多端环境中继续 smoke。
- GUI/VSCode 插件仍需要消费相同 Ask/Plan app-server 协议并处理 unsupported fallback。
- Ask UI 和 Plan panel 仍有体验优化空间，适合作为 C4 的“体验优化 + 发布”主题继续推进。
