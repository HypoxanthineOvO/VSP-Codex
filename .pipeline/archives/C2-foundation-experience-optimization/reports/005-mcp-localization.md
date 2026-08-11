# M5 MCP 提示迁移与静态汉化报告

## 结果

M5 已完成。普通 MCP startup `Starting` 状态不再占用主 Working/status header，因此 `Booting MCP server: context7` 这类正常启动噪声不会挡住用户当前工作。MCP 配置、工具和失败路径没有删除；失败/中断仍会在历史中显示关键技术细节。

同时完成第一轮静态汉化：Slash Command descriptions、设置页高频内容、status line/terminal title 配置页、部分 welcome/status/error/snapshot 文案。专业术语、命令名和关键配置词保留英文。

## 主要改动

- `codex-rs/tui/src/chatwidget/mcp_startup.rs`：普通 `Starting` 不再拥有主 status header；failure warning 保持可见。
- `codex-rs/tui/src/chatwidget/tests/mcp_startup.rs`：增加 MCP startup 降噪回归测试，确认普通启动保留 Working，失败仍显示 warning。
- `codex-rs/tui/src/slash_command.rs`：Slash Command description 汉化，并修正 `/feedback` 为 VSP 禁用语义。
- `codex-rs/tui/src/chatwidget/settings_popups.rs`：设置页第一轮汉化。
- `codex-rs/tui/src/bottom_pane/status_line_setup.rs`、`title_setup.rs`：状态栏和终端标题配置页第一轮汉化。
- `codex-rs/tui/src/app/session_lifecycle.rs`、`app/agent_navigation.rs`：Subagent picker 空状态和说明改为中文。
- `codex-rs/tui/src/bottom_pane/feedback_view.rs`：禁用反馈入口的备用弹窗文案改为 VSP 语义。

## 测试与验证

- `just test -p codex-tui mcp_startup`：17 passed。
- `just test -p codex-tui slash_command`：138 passed。
- `just test -p codex-tui`：2646 passed, 4 skipped。
- follow-up focused tests：`feedback_selection_popup_is_disabled_in_vsp open_agent_picker_shows_empty_state_when_collab_is_disabled core_command_descriptions_are_localized_to_chinese disabled_official_service_descriptions_do_not_advertise_upload picker_subtitle_mentions_shortcuts`：5 passed。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。
- `just fix -p codex-tui`：通过；后续只做了 M5 follow-up 的 focused 测试。

## Worker 证据

- test worker：`019e60d6-f692-7cf1-8eb5-1b632f559064`，补充 MCP startup 和 Slash Command 汉化红测。
- implement worker：`019e60df-9edf-7bf2-b55e-2f4a50e4e58a`，完成 MCP startup 降噪和第一轮静态汉化。
- audit worker：`019e60fa-d2b5-7120-aa48-963be5e63e9d`，指出两个问题：VSP 行为下的 UI snapshot 覆盖不足，以及 `/feedback` description 误导为上传日志。
- audit follow-up：`019e6106-c8a4-7a42-9ef1-85505cb48710`，复核 follow-up 后结论 pass。

## 已知说明

- 本轮是高频路径第一轮静态汉化，仍会有低频内部提示或测试 fixture 保留英文。
- 旧的 `multi_agent_enable_prompt` snapshot 被移除是预期行为：M2 已让 Subagent 默认启用，M5 follow-up 改为覆盖当前 VSP 语义下的空状态和“不弹启用 prompt”行为。
- `/feedback` 官方上传服务仍禁用；本轮只保留禁用提示和相关回归覆盖，不恢复官方反馈上传流程。
