# C7 Experience Optimization And Bugfix Sweep

时间：2026-05-29T21:17:55+08:00

## 范围

C7 是一次用户可见体验修复 Cycle，覆盖长会话恢复、终端 scrollback、Subagent 身份与报告可读性、工具输出翻译边界、Workflow 结构化交互、Plan/SubPlan live surface、VSP 更新发布通道和外观入口。

## 架构方向

- TUI 稳定性优先：M1 聚焦 resume/replay/reflow/scrollback 清屏行为，避免用大重构掩盖实际 flicker/jump 问题。
- Subagent UX 分层：M2 先统一身份解析、名字租约和角色/effort 策略；M4 再做 Prompt/Report Markdown viewer，避免把调度语义和内容查看混在一起。
- 翻译边界前置：M3 先停止工具/状态输出的盲目 append translation，M4 的显式翻译 viewer 建立在这个边界之上。
- Workflow 交互结构化：M5 将 required decisions 统一到 `request_user_input` 卡片；真实权限仍由 `request_permissions` 或 approval overlay 控制。
- Plan live surface 依赖交互卡片：M6 让 Plan/SubPlan 成为 Workflow 实时状态面，并在计划确认/阶段转换中复用 M5 的结构化卡片。
- 发布通道与名称元数据解耦：M7 将内部/外部 Subagent 默认名字作为 release-channel metadata，而不是 semver/tag 语义。
- 外观入口收尾：M8 保留 `[tui].theme` 与 `[tui].ui_theme` 的配置分离，用户入口合并到 Appearance/Theme。

## Milestone Map

1. M1 Resume And Scrollback Stability
2. M2 Subagent Identity, Role Policy, And Name Leases
3. M3 Tool And Collaboration Output Rendering Boundaries
4. M4 Subagent Prompt And Report Viewer
5. M5 Structured Interaction Gates And Authorization Cards
6. M6 Workflow Plan/SubPlan Live Surface
7. M7 VSP Release Channels, Naming Metadata, And Updater
8. M8 Appearance Entry Point And Final UX Polish

## 结构化交互卡片类型

M5 至少固化以下卡片类型：

- Plan confirmation card
- Phase transition card
- Scope selection card
- Revision/rejection card
- Authorization-intent card
- Release/update confirmation card

确认卡只表达用户意图或工作流决策；不能替代真实权限授权。网络、破坏性操作、系统级依赖安装、远程发布等仍必须走对应权限/approval gate。

## 验证约束

- 每个实现 Milestone 保持 strict worker separation：`test`、`implement`、`audit`。
- `test` 负责红测、fixtures、assertions、snapshots 和最终验证证据。
- `implement` 不得创建、编辑、重写或满足 test-owned assets。
- `audit` 必须只读，并审查证据质量、边界、风险和角色分离。
- P4 前必须完成 Subagent prompt audit，审查 8 个 prompt 是否保留 P2 技术路线、真实验证路径、安全门禁和 M5 卡片语义。
