# C7/M3 Tool And Collaboration Output Rendering Boundaries

完成时间：2026-05-30T03:41:55+08:00

## 结果

M3 已完成。Exec、MCP 和 collaboration lifecycle/status 输出不再被盲目 append 翻译；技术输出、命令、路径、ID、YAML/Markdown 片段会保持 literal。Reasoning/prose 的翻译路径仍保留，避免把翻译能力全局关掉。

## 主要变更

- `wrap_translated` 现在只对 `ReasoningSummaryCell` 应用 `TranslatedHistoryCell`。
- completed `ExecCell` 输出不再走 append translation，保留原始工具输出。
- completed `McpToolCallCell` 不再暴露 tool-output translation source，显示/raw 内容仍从 literal tool result 渲染。
- `wrap_collab_translated` 改为返回原始 `PlainHistoryCell`，让 `Waiting for ...`、`Ran ...` 等生命周期/状态行不再出现翻译副本。
- `TranslationScope` / `TranslationDisplay` 收窄到当前真实使用的 `Reasoning` / `Replace`。

## 验证

- `cargo test -p codex-tui translated_wrapping_skips_completed_exec_tool_output -- --nocapture`：通过。
- `cargo test -p codex-tui translated_wrapping_skips_completed_mcp_tool_output -- --nocapture`：通过。
- `cargo test -p codex-tui collab_lifecycle_status_lines_are_not_append_translated -- --nocapture`：通过。
- `cargo test -p codex-tui reasoning_summary_still_uses_translation_wrapper -- --nocapture`：通过。
- `cargo test -p codex-tui hypo_translate -- --nocapture`：8 项通过。
- `cargo test -p codex-tui translated -- --nocapture`：7 项通过。
- `cargo fmt --check`：通过；仅有既有 stable rustfmt 对 nightly-only `imports_granularity = Item` 的 warning。
- `git diff --check`：通过。

## 审计

只读 audit worker 孟轲 the 10th 返回 `pass`。审计确认 M3 修复关闭了 exec/MCP/collab 盲目翻译问题，测试覆盖真实失败模式，并保留 reasoning/prose 翻译正向 guard。

## 非阻塞项

- `cargo test -p codex-tui history_cell -- --nocapture` 仍有一个独立 snapshot 版本漂移：`session_info_availability_nux_tooltip_snapshot` 期望 `v0.134.0-vsp.4`，当前 crate 版本为 `0.134.0-vsp.5`。该问题与 M3 翻译边界无关，留给后续体验/版本收口处理。
- M3 不实现 Subagent Prompt/Report Markdown viewer；该工作进入 C7/M4。
