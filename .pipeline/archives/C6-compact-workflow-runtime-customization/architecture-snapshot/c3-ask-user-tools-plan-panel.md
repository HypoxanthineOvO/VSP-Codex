# C3 架构计划：Ask User Tools 与 Plan 常驻窗口

生成时间：2026-05-26T18:20:00+08:00

## 目标

C3 实现两条一等主线：

- Ask User Tools：把现有 `request_user_input` 升级为支持多题型、结构化答案、跨客户端 native UI、unsupported fallback 和跨设备 resume 的协作原语。
- Plan/SubPlan 常驻窗口：把现有 `update_plan` 从 flat checklist 演进为 structured Plan/SubPlan state，并在 TUI 常驻显示，同时通过 app-server v2 暴露给未来 VSCode/GUI。

## 关键决策

- Ask 基于现有 `request_user_input`，不新增平行工具族。
- Ask 可用于默认 bypass/ordinary interactive turn，不限于 Plan mode。
- Subagent 不直接弹 Ask；必须经 root session。
- Ask 支持 request-level summary 和 per-question description。
- Ask 题型覆盖 single_choice、multi_choice、short_answer/free_text、order-only ranking。
- app-server Ask API 在 C3 保持 experimental；native Ask UI support 是独立 capability，不等同于 `experimentalApi`。
- Pending Ask 必须跨设备 resume；unsupported client 降级为普通编号聊天问题。
- Plan/SubPlan 改造现有 `update_plan`，保留 legacy flat checklist 兼容。
- Plan/SubPlan state 是 canonical state，不从 proposed-plan markdown/history cell 解析。
- SubPlan 首版只支持一层。
- TUI Plan panel 平时常驻，Default idle 折叠为一行摘要，并支持快捷键展开。
- C3 最终验收必须包含用户真实换电脑 resume pending Ask；失败则本 Cycle extend milestone 回修。

## Milestone 顺序

1. M1 Ask 协议模型与 app-server experimental schema
2. M2 Core Ask 工具语义与模型契约
3. M3 TUI Ask 多题型与隐私展示
4. M4 Pending Ask resume 与 unsupported-client fallback
5. M5 Plan/SubPlan structured state 与 app-server v2 暴露
6. M6 TUI 常驻 Plan/SubPlan 面板
7. M7 集成隐私回归与真实跨设备验收

## 验证基线

- app-server protocol/schema: `just write-app-server-schema --experimental` 和 `just test -p codex-app-server-protocol`
- core behavior: focused `just test -p codex-core ...`
- TUI behavior: `just test -p codex-tui` 和 `cargo insta pending-snapshots -p codex-tui`
- Rust hygiene: `just fmt` 和 scoped `just fix -p <changed-crate>`
- Final acceptance: user real cross-device pending Ask resume test.

## 产物

- P2 technical route: `.pipeline/plan-state/technical-route.md`
- P2 machine plan: `.pipeline/plan-state/decompose.yaml`
- Prompts: `.pipeline/prompts/001-*.md` through `.pipeline/prompts/007-*.md`
