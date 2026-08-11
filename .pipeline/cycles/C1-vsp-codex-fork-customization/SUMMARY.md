---
archived_source: .pipeline/archives/C1-vsp-codex-fork-customization
cycle: C1-vsp-codex-fork-customization
kind: cycle-summary
started: 2026-05-24T19:40:25+08:00
status: closed
---

# vsp codex fork customization 总结

## 目的与边界

- 状态：completed
- 开始时间：2026-05-24T20:16:00+08:00
- 关闭时间：2026-05-25T19:29:18+08:00
- 类型：build
- Preset：strict worker separation / regression-first
- 归档路径：`.pipeline/archives/C1-vsp-codex-fork-customization/`

## 最终结果

旧 Cycle 状态：`m6_completed_awaiting_review`。本预览不重新判断旧结果，只建立可读导航。

## 验证结果

- 6 个历史 milestone 已映射。
- 60 个旧文件继续保存在 `.pipeline/archives/C1-vsp-codex-fork-customization`。

## 重要决定与经验

- 旧 `cycle.yaml` 没有结构化 lessons；请查看原始 Summary。

## 后续候选

- 不自动继承旧任务；新 Cycle 应通过 `builds_on` 选择需要的结果或经验。

## 映射缺口

- 无结构性缺失。

## 旧总结原文

# Cycle C1: VSP Codex Fork Customization

- 状态：completed
- 开始时间：2026-05-24T20:16:00+08:00
- 关闭时间：2026-05-25T19:29:18+08:00
- 类型：build
- Preset：strict worker separation / regression-first
- 归档路径：`.pipeline/archives/C1-vsp-codex-fork-customization/`

## 改动摘要

本 Cycle 将官方 Codex fork 收敛为实验室内部使用的 `VSP Codex`：新增 `vsp-codex` 外部入口，默认使用 `~/.vsp-codex`，共享原 Codex sessions/skills/plugins 资产，提供 Base URL + API Key 的 VSP Provider 初始化和 `/provider` 切换入口，隐藏官方 Cloud/OAuth/remote-control/update/feedback 等入口，并清理 TUI 中主要官方品牌提示。

## 技术思路

实现策略以降低后续合并官方更新成本为主：保留底层官方代码和 crate 结构，在 CLI/TUI/app-server 边界集中 gate 官方服务入口；Provider 能力复用现有 `model_providers` 与 Responses API wire；共享资产通过 home-dir 层处理，避免复制完整 `~/.codex` 配置和认证文件；UI 变化用现有 ratatui/snapshot 测试覆盖。

## Milestones

| Milestone | 结果 |
|---|---|
| M1 | 建立当前测试基线，修复环境假设导致的 TUI 失败，并确认 `codex-tui` 本地绿色。 |
| M2 | 新增 TUI 用户模拟回归测试和 GitLab CI 收敛，M2 已提交并推送，pipeline #136 通过。 |
| M3 | 新增 `vsp-codex` 入口和 VSP home，M3 已提交并推送，pipeline #137 通过。 |
| M3.5 | 将 `~/.vsp-codex` 共享官方 Codex sessions/skills/plugins 资产，但隔离 config/auth/provider。 |
| M4 | 改造首启 Provider 初始化页，新增 `/provider`，并让当前 thread/provider 设置同步到 app-server。 |
| M5 | 隐藏或禁用官方服务入口，保留 MCP、Skills、Plugins、OpenAI-compatible Provider 和 `exec-server --remote`。 |
| M5.5 | 修复 `resume` 在共享 sessions 下被 legacy provider metadata 过滤的问题。 |
| M6 | 完成 VSP 品牌、官方提示和主要 TUI 文案清理。 |
| M6.1 | `/provider` 隐藏内置 Provider，并将版本设为 `0.134.0-vsp.1`。 |

## 修改文件 / 模块

- CLI/包装层：`codex-rs/cli/`、`codex-cli/`、安装脚本和测试入口。
- Home/config：`codex-rs/utils/home-dir/`、`codex-rs/core/src/config/`。
- Provider/app-server：`codex-rs/tui/src/vsp_provider.rs`、`codex-rs/tui/src/chatwidget/provider_popups.rs`、`codex-rs/app-server*`、`codex-rs/protocol*`。
- TUI：onboarding、slash command、status、session header、plugins/app-link、resume picker、tooltips 和 snapshots。
- Release/CI：`.gitlab-ci.yml`、`Cargo.toml`、`Cargo.lock`、`README.md`、`CHANGELOG.md`。
- Workflow 证据：`.pipeline/reports/`、`.pipeline/reviews/`、`.plan-state/`。

## 测试设计

测试按风险分层：先建立完整 `codex-tui` baseline，再为 TUI 用户行为补 snapshot/user-simulation；Provider 切换用 app-server thread settings 和真实 `/responses` base_url 回归覆盖；官方入口清理使用 CLI/TUI focused tests；UI 品牌变化使用 snapshot；发布前执行 `just fmt`、focused crate tests、`cargo build --bin vsp-codex`、snapshot pending 检查和 CI lint。

## 验证结果

关键验证已通过：

- `just test -p codex-tui`: 通过（M1/M2/M3.5 阶段完整跑通）。
- `just test -p codex-app-server thread_settings_update`: 通过。
- `just test -p codex-app-server-protocol`: 通过。
- `just test -p codex-cli official_service_cli_commands_are_disabled`: 通过。
- `just test -p codex-cli vsp_codex_help_uses_vsp_codex_usage`: 通过。
- `just test -p codex-tui user_typing_provider_slash_command_opens_provider_picker`: 通过。
- `just test -p codex-tui vsp_local_picker_does_not_filter_by_current_provider`: 通过。
- `cargo build --bin vsp-codex`: 通过。
- `just fmt`: 通过。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`: no pending snapshots。
- `git diff --check`: 通过。
- `/home/heyx/.local/bin/vsp-codex --version`: `vsp-codex 0.134.0-vsp.1`。

## 预期结果

用户可以在任意仓库运行 `vsp-codex`，默认读取 `~/.vsp-codex`，继续看到共享的 Codex sessions/skills/plugins，首启以 VSP Provider 初始化替代 OAuth/API Key 官方选择页，`/provider` 只显示 VSP 和用户自定义 Provider，普通官方服务入口不会再出现在常规 UI/help 中。

## 遇到的问题

- 原始 TUI 测试失败来自测试环境假设，不是功能改动；已修复 `/tmp` git 影响和 IPC socket tempdir 权限问题。
- Python SDK 默认 runtime wheel 在当前 Linux glibc 环境缺失；已让 `just fmt` 避开 SDK runtime 安装，但 SDK 默认安装问题留作后续。
- 并行 Cargo 测试会争抢 build lock，后续用顺序 focused tests 避免误判。

## 风险 / 后续

- 官方服务底层实现仍保留，只在入口处隐藏/禁用；这是刻意选择，用于降低后续 upstream merge 成本。
- SDK/runtime 发布模型仍未纳入本 Cycle。
- `exec-server --remote` 按用户要求保留，未来如需完全去官方依赖，需要单独设计 VSP registry/rendezvous。
- 后续可开启新 Cycle 做更全面中文本地化、Provider 编辑 UI、官方底层代码删除或 SDK 发布修复。
