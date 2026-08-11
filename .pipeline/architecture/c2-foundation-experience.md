# C2 架构计划：基础体验优化

生成时间：2026-05-25T23:42:15+08:00

## 目标边界

C2 面向 VSP Codex 的基础体验优化，不处理 C1 已关闭的 Provider、Session 迁移、官方 Cloud/OAuth 清理主线。C2 的范围是默认运行策略、Subagent 体验、动态翻译、静态汉化、MCP 启动提示降噪和集成回归。

## 计划结构

1. M1 默认 YOLO 与底栏 Context：所有 `vsp-codex` 本地入口默认无审批全权限运行，并在底栏展示 Context Window。
2. M2 Subagent 核心策略：移除显式授权软限制，默认鼓励使用；上限 32；中文名字池；保留 model/reasoning effort 指定能力。
3. M3 Subagent 常驻卡片与键盘查看：展示活跃 Subagent 列表，支持查看命令、返回内容和状态详情。
4. M4 Hypo-Translate 运行时翻译：接入本机 `http://127.0.0.1:8998`，翻译 reasoning、Subagent transcript、tool summaries/returns。
5. M5 MCP 提示迁移与静态汉化：普通 MCP booting 提示离开主 Working 区；高频 TUI/Slash Command/settings/error 文案汉化。
6. M6 集成回归与实际验收：验证上述体验可以组合运行，并准备人工验收。

## 架构原则

- 默认策略优先放在 VSP entrypoint/config normalization 层，避免误伤官方兼容路径。
- Subagent 名字是 display metadata，内部身份继续使用稳定 id。
- TUI 新展示区域使用结构化状态，不从 transcript 文本中解析。
- 动态翻译是非阻断能力；服务不可用时 passthrough 原文。
- MCP 功能不删除，只迁移普通成功启动提示的位置；失败仍需显眼。
- 所有用户可见 UI 变更需要对应 `insta` snapshot 覆盖。

## 验证基线

- Rust 代码变更后运行 `just fmt`。
- 按改动 crate 运行 scoped `just test -p <crate>`。
- TUI 文案/布局变更运行 `just test -p codex-tui` 和 `cargo insta pending-snapshots -p codex-tui`。
- 如 core/shared/protocol 改动过大，运行完整 `just test` 前先向用户确认。

## 执行策略

用户已要求 C2 不设置中途 Gate。之后输入 Start 即视为执行确认；执行应按 M1 到 M6 连续推进，除非遇到真实阻塞、重复验证失败、缺少外部输入，或到达最终人工验收点。
