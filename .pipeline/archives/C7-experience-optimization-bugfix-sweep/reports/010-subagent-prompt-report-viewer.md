# C7/M4 Subagent Prompt And Report Viewer

完成时间：2026-05-30T04:59:13+08:00

## 结果

M4 已完成。Active Subagents `Ctrl+O` 详情现在提供显式 Prompt/Report viewer，能够查看当前选中 Subagent 的完整 prompt 和 report/message，并以 Markdown 渲染。普通 active row 和 lifecycle rows 仍保持简洁，不泄漏完整 prompt/report。

## 主要变更

- `Ctrl+O` Active Subagents popup 增加 Prompt/Report viewer surface。
- viewer 使用现有 Markdown renderer 渲染完整原文，避免把 raw triple-backtick fence 直接暴露为普通文本。
- viewer 显示 `Original / 原文` 和 `Translated / 翻译` 标签；翻译尚未生成时明确标注“未生成，同原文预览”。
- 窄宽度下使用 stacked viewer，保留 Prompt/Report、原文/翻译标签、核心 heading 和完整 tail marker。
- 多 active subagent 时，viewer 通过 `on_selection_changed` 和共享 selected index 跟随当前选中行，而不是固定第一行。

## 验证

- `cargo test -p codex-tui c7_m4 -- --nocapture`：4 项通过。
- `cargo fmt --check`：通过；仅有既有 stable rustfmt 对 nightly-only `imports_granularity = Item` 的 warning。
- `git diff --check`：通过。

## 审计

首轮只读 audit 返回 `needs_changes`，指出多 active subagent 时 viewer 固定显示第一行。修复轮补充 selected-row 红测并实现 selection 绑定后，复审 worker 孔丘 the 13th 返回 `pass`，无阻塞发现。

## 非阻塞项

- viewer 内容在 popup 打开时快照捕获；如果 prompt/report 在 popup 已打开期间继续更新，需要重新打开 popup 才能看到新内容。
- 长不可断 token 为保留完整原文和可检索性，窄屏可能视觉溢出。
- 翻译视图当前为清晰标注的未生成占位，尚未接入实际 lazy translation。
