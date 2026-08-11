# C7/M4 Audit Review

## Verdict

pass

## Findings

无阻塞发现。

## Review Scope

- `codex-rs/tui/src/chatwidget/active_subagents.rs`
- `codex-rs/tui/src/chatwidget/tests/app_server.rs` 的 `c7_m4` focused tests
- `.pipeline/reviews/C7/M4/test.md`
- `.pipeline/reviews/C7/M4/implement.md`

## Initial Audit Finding

首轮审计返回 `needs_changes`：Active Subagents `Ctrl+O` Prompt/Report viewer 固定使用 `active_rows().first()`，多 active subagent 时，即使列表选中第二行，viewer 仍显示第一名的 prompt/report。

## Recheck Result

复审确认 selected-row 修复真实解决了多 active subagent 问题：

- `open_active_subagents_detail_popup` 从同一 `active_rows` 顺序构建 list items 和 viewer entries。
- `SelectionViewParams.on_selection_changed` 更新共享 selected index。
- viewer render 时按当前 selected index 取 entry。
- `c7_m4_active_subagents_ctrl_o_viewer_tracks_selected_row` 覆盖了 Down 到第二个 subagent 后 viewer 只显示 Ada 的 prompt/report marker。
- full prompt/report 仍只在 `Ctrl+O` viewer 中显示；active row、detail preview 和 lifecycle rows 仍保持简洁。
- Markdown fence、未生成翻译标签和窄屏 stacked viewer 行为均有 focused tests 覆盖。

## Validation

- `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-tui c7_m4 -- --nocapture`
  - 通过：4 passed / 0 failed。
  - 仅观察到既有 `TurnContext::cwd` deprecated warning。
- `cd /home/heyx/Codex-VSP/codex-rs && cargo fmt --check`
  - 通过，退出码 0。
  - 仅观察到既有 stable rustfmt 对 nightly-only `imports_granularity = Item` 的 warning。
- `cd /home/heyx/Codex-VSP && git diff --check`
  - 通过，无输出。

## Residual Risks

- viewer entries 在 popup 打开时快照捕获；popup 打开期间新到的 prompt/report 更新需要重新打开 popup 才能看到。
- 很长的不可断 token 会为保留可检索原文而在窄终端里视觉溢出。
- 翻译视图当前是明确标注的“未生成，同原文预览”，不是实际 lazy machine translation。
