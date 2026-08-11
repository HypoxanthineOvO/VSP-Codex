# Codex-VSP 项目摘要

本文件为 Hypo-Workflow 自动生成的跨 Cycle 项目摘要。权威文件位于 `.pipeline/`，修改后运行 `/hw:sync --repair`。

- 当前 Cycle：C10 `Upstream Sync And VSP Client Connectivity Adaptation`（active，2026-07-02 开始）
- 已发布基线：`v0.140.0-vsp.8.1`（GitLab），本地热修至 `v0.140.0-vsp.8.2`
- 上游收敛基线：`rust-v0.143.0-alpha.25` 等价收敛（非完整合并）

## Cycle 一览

| Cycle | 名称 | 类型 | 状态 | 开始时间 | 亮点 |
| --- | --- | --- | --- | --- | --- |
| C1 | VSP Codex Fork Customization | feature | 完成 | — | Fork 定制与 VSP 基础表面建立 |
| C2 | Foundation Experience Optimization | feature | 完成 | 2026-05-25 | 基础体验优化 |
| C3 | Ask User Tools Plan Panel | feature | 完成 | 2026-05-26 | Question Tool 与 Plan 面板 |
| C4 | Experience Optimization Release | feature | 完成 | 2026-05-27 | 体验优化并发布 |
| C5 | Experience Optimization Continuation | feature | 完成 | 2026-05-28 | 体验优化续作 |
| C6 | Compact Workflow Runtime Customization | feature | 完成 | 2026-05-29 | Compact 与 Workflow 运行时定制 |
| C7 | Experience Optimization Bugfix Sweep | bugfix | 完成 | 2026-05-29 | 缺陷清扫 |
| C8 | VSCode Remote SSH Session Compatibility And Plugin UX | feature | 完成 | 2026-05-30 | Remote SSH 会话兼容与插件 UX |
| C9 | Deep Research Built-In And Codex Fork Assessment | feature | 完成 | 2026-06-14 | 上游升级至 alpha.18 + alpha.25 等价收敛，发布 v0.140.0-vsp.8.1，Deep Research 需求简报 |
| C10 | Upstream Sync And VSP Client Connectivity Adaptation | feature | 进行中 | 2026-07-02 | 同步上游最新版本；修复 VSCode Plugin/Desktop 等客户端连接适配 |

## Open Patches

- 无（P001 已关闭）

## Deferred Items

- 无（C1–C9 归档均无延后事项）

## 跨 Cycle 风险与关注点

- 全量 workspace nextest 在本机保持历史性红色（libcap.pc 缺失、app-server 集成套件超时）；发布验证依赖 focused tests + smoke。
- VSCode Remote extension 自动更新会重新引入官方 bundled `codex`；当前依赖本地 repair timer 热修缓解（2026-06-27 debug 记录），C10 计划将其固化为版本化行为。
- Deep Research 目前仅为 design-only 需求简报（C9/M6），未实现生产行为。
