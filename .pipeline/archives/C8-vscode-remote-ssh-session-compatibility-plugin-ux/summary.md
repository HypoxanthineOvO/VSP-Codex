# C8 归档摘要

- Cycle：C8 `VSCode Remote SSH Session Compatibility And Plugin UX`
- 类型：feature
- 状态：completed
- 开始时间：2026-05-30T13:37:59+08:00
- 完成时间：2026-05-30T22:44:30+08:00
- 归档整理时间：2026-06-14T08:44:37+08:00
- Preset：tdd

## Milestones

- M1 Remote SSH VSCode Integration Probe：验证并修正 Remote SSH server Machine settings 优先级。
- M2 VSCode Plugin Protocol Capability Map：调研插件协议能力和不 fork 插件边界。
- M3 Provider And Resume Compatibility Contract：修复空 provider metadata 导致的插件 resume 失败。
- M4 Session Discovery Strategy For Official Codex：增加官方 Codex discovery 的脱敏 export 策略。
- M5 Request Tool And Request Input UX For Plugin：透传显式 Workflow metadata 并增加 Markdown fallback。
- M6 Markdown Workflow Surface And Remote SSH Acceptance：将 Request Input fallback 接入 live plugin-facing thread output。

## 关键数据

- 报告归档：`.pipeline/archives/C8-vscode-remote-ssh-session-compatibility-plugin-ux/reports/`
- Prompt 归档：`.pipeline/archives/C8-vscode-remote-ssh-session-compatibility-plugin-ux/prompts/`
- Review 归档：`.pipeline/archives/C8-vscode-remote-ssh-session-compatibility-plugin-ux/reviews/C8/`
- State 归档：`.pipeline/archives/C8-vscode-remote-ssh-session-compatibility-plugin-ux/state.yaml`
- PROGRESS 归档：`.pipeline/archives/C8-vscode-remote-ssh-session-compatibility-plugin-ux/PROGRESS.md`
- 知识摘要：`.pipeline/archives/C8-vscode-remote-ssh-session-compatibility-plugin-ux/knowledge-summary.md`
- 延后事项：0

## 完成说明

改动摘要：C8 完成 VSCode Remote SSH session 兼容、provider resume 修复、官方 session discovery 策略、Request Input UX 和 Markdown Workflow fallback。

技术思路：优先服务端和协议兼容，默认不 fork VSCode 插件；对官方 Codex discovery 坚持脱敏、导出和显式授权边界。

修改文件/模块：历史归档整理移动 `.pipeline` 运行态产物到 `.pipeline/archives/C8-*`，业务代码未在本整理步骤中修改。

测试设计：原 Cycle 使用 app-server、rollout、CLI 和 TUI focused tests，并通过只读 audit 复核。

验证结果：根据归档报告，`request_user_input`、`workflow_markdown_surface`、`thread_resume`、`official_codex_discovery`、`codex-cli --test vscode`、fmt 和 diff checks 在 C8 收尾时通过。

预期结果：C8 历史现在完成归档，根 `.pipeline` 运行区可用于新的 C9 Cycle。

遇到的问题：C8 已 completed 但未归档，且后续 release/maintenance 记录继续写入根运行区；本次将 C8 本地运行态归档，保留持久 log/knowledge/release 文件原位。

风险/后续：Remote SSH manual acceptance 仍是 checklist artifact，不应误写为真实外部发布验收已完成。
