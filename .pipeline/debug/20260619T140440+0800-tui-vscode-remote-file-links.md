# TUI VSCode Remote File Links Debug Report

## 改动摘要

症状：VSCode Remote SSH 终端中 Ctrl+单击 TUI 渲染出的本地文件链接时，`file:///home/...` 被 Windows 本地 opener 处理，导致 “系统找不到指定的文件”。不能退回到 VSCode 自动路径识别，因为自动识别会把路径前面的中文文本一并纳入 clickable range。

修复：保留 TUI 现有 `HyperlinkLine.columns` 精确范围和 OSC8 输出，在 VSCode integrated terminal 且目标为 `file://` 时，将 OSC8 destination 从本地 `file:///abs/path#Lx` 改写为 VSCode Remote URI：`vscode-remote://<authority>/abs/path#Lx`。非 VSCode 终端、非文件链接、以及无法发现 remote authority 时保持原行为。

同时将仓库根目录 `tmp/` 加入 `.gitignore`，避免截图/临时图片进入版本控制。

## 技术思路

根因判断：

1. 现有 TUI 已经将 visible text 和 hyperlink range 分离，`decorate_spans()` / `mark_buffer_hyperlinks()` 用 OSC8 包裹精确列范围。
2. 旧的临时方案若禁用 `file://` OSC8，会退回 VSCode 的自动路径检测，复发“中文前缀被识别进链接”的问题。
3. VSCode Server 日志显示远端文件内部使用 `vscode-remote://ssh-remote+.../home/...` URI；因此正确修复点是保留 OSC8 范围，只改 OSC8 destination。

实现细节：

- 新增 `terminal_osc8_destination()` 作为 OSC8 输出层的目标转换入口。
- `terminal_hyperlink_destination()` 仍只负责语义安全过滤，保留已存储的 `file://` destination。
- 仅当 `TERM_PROGRAM=vscode` 或存在 `VSCODE_IPC_HOOK_CLI` 且目标 scheme 为 `file` 时，才懒加载 VSCode Remote authority。
- authority 优先来自 `VSP_CODEX_VSCODE_REMOTE_AUTHORITY`，否则从 `~/.vscode-server/data/logs/**/*.log` 的最近日志尾部解析 `vscode-remote://...`。
- 保留 fragment/query，确保 `#L74` 这类行号跳转信息不丢失。

## 修改文件/模块

- `.gitignore`
- `codex-rs/tui/src/terminal_hyperlinks.rs`
- `codex-rs/tui/src/markdown_render_tests.rs`
- `.pipeline/debug/20260619T140440+0800-tui-vscode-remote-file-links.md`

## 测试设计

新增/调整覆盖：

- VSCode integrated terminal 中 `file://` OSC8 destination 会改写为 `vscode-remote://...`。
- 无 remote authority 时保持普通 `file://`，避免误删链接。
- 能从 VSCode Server 日志文本解析 percent-encoded authority。
- 带中文前缀的 Markdown 本地文件链接只覆盖路径文本，不覆盖 `输出文件： ` 前缀。

Worker separation：本项目配置为 strict，但本轮没有收到显式子代理/并行工作授权，且当前 multi-agent 工具说明要求只有用户明确要求 subagents/delegation/parallel agent work 时才能 spawn。因此本轮修复、测试和审计由主 Agent 本地完成，不能声称满足独立 worker separation。

## 验证结果

通过：

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui terminal_hyperlinks -- --nocapture`
  - 9 passed, 0 failed
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui file_link_hyperlink -- --nocapture`
  - 2 passed, 0 failed
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`
  - passed；仅 stable rustfmt 对 unstable `imports_granularity` 的既有 warning
- `git diff --check`
  - passed

既有 warning：

- `codex-core` 中若干 dead_code warning
- `codex-tui` 中 `RestartRealtimeAudioDevice.kind` 与 `TableCell::patch_style` dead_code warning

## 预期结果

在 VSCode Remote SSH integrated terminal 中，TUI 文件链接仍只有路径文本区域可点击；点击后应由 VSCode 以 remote URI 打开远端 `/home/...` 文件，而不是交给 Windows 本地 `file://` opener。

## 遇到的问题

- 初始修复方向曾尝试在 VSCode 中禁用 `file://` OSC8，已根据用户反馈废弃，因为这会触发 VSCode 自动路径识别并污染中文前缀范围。
- VSCode 没有在普通终端环境变量中直接暴露 remote authority，本修复采用 env override + VSCode Server 日志解析的组合方案。
- Worker separation 降级，见“测试设计”。

## 风险/后续

- 如果 VSCode Server 日志中没有任何 `vscode-remote://` URI，文件链接会安全降级为原 `file://`，但该环境下仍可能复现 Windows 本地 opener 问题。可通过设置 `VSP_CODEX_VSCODE_REMOTE_AUTHORITY` 显式覆盖。
- 尚未打包发布；若要让插件用户拿到修复，需要后续执行同版本 HotFix 构建、CI、Release asset 替换。
