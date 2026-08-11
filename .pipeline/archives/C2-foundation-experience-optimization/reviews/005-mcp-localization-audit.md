# M5 MCP 提示迁移与静态汉化审核

## Verdict

pass

## Reviewed Refs

- `codex-rs/tui/src/chatwidget/mcp_startup.rs`
- `codex-rs/tui/src/chatwidget/tests/mcp_startup.rs`
- `codex-rs/tui/src/slash_command.rs`
- `codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs`
- `codex-rs/tui/src/app/session_lifecycle.rs`
- `codex-rs/tui/src/app/agent_navigation.rs`
- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/bottom_pane/feedback_view.rs`
- M5 localization snapshots

## Findings

初始 M5 审计发现两个问题，均已修复：

- P2：`feedback_selection_popup.snap` 和 `multi_agent_enable_prompt.snap` 的旧覆盖被删除后，没有用当前 VSP 语义补回 UI 回归覆盖。已增加 `/feedback` 禁用消息/弹窗 snapshot，以及 Subagent picker 禁用 collab 时的空状态 snapshot，并断言不会发出 feature update prompt。
- P3：`/feedback` description 写成“发送日志给维护者”，但 VSP 中官方反馈入口已禁用。已改为“VSP 中已禁用官方反馈入口”，并补充单元测试。

复核审计结论：

- `/feedback` 不再宣称上传日志，dispatch 路径显示 VSP 禁用提示。
- MCP `Starting` 普通状态不再污染主 Working/status header。
- MCP failure warning 仍保留关键技术信息。
- Slash Command 和设置/status/title 第一轮汉化保留了 `MCP`、`provider`、`Base URL`、`Plan mode`、`raw`、`model`、`reasoning effort`、`status line`、`terminal title` 等术语。

## Residual Risks

- 低频 TUI 文案、插件市场深层文案、部分内部 error fixture 仍可能是英文；这属于后续汉化增量，不阻塞 M5。
- snapshot diff 中包含 `Context` 和版本号变化，来源于 M1/C1 相关改动，不属于 M5 阻塞项。

## Validation

- `just test -p codex-tui mcp_startup`：17 passed。
- `just test -p codex-tui slash_command`：138 passed。
- `just test -p codex-tui`：2646 passed, 4 skipped。
- M5 follow-up focused tests：5 passed。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。
- `just fix -p codex-tui`：通过。
