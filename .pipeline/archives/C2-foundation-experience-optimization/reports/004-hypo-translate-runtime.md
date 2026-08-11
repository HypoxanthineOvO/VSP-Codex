# M4 Hypo-Translate 运行时翻译报告

## 结果

M4 已完成。VSP Codex 现在包含独立的 Hypo-Translate 运行时翻译模块，默认连接 `http://127.0.0.1:8998`，优先使用 `/translate/stream`，失败时降级到 `/translate`，服务不可用时保持原文显示且不阻塞 TUI。

翻译范围只接入指定的动态内容：reasoning/思维输出、Subagent 协作 transcript、tool summaries/returns。静态 UI 文案仍由 M5 处理。

## 主要改动

- `codex-rs/tui/src/hypo_translate.rs`：新增 Hypo-Translate HTTP/SSE 客户端、超时、fallback、事件解析和测试。
- `codex-rs/tui/src/history_cell/base.rs`、`messages.rs`、`mcp.rs`、`mod.rs`：新增 translated history-cell wrapper，显示层可异步替换译文，raw/original 路径仍保留原文。
- `codex-rs/tui/src/chatwidget.rs`、`constructor.rs`、`tool_lifecycle.rs`：接入非阻塞 runtime translator，并在指定动态 history cell 上触发翻译。
- `codex-rs/tui/src/app_event.rs`、`app/event_dispatch.rs`：翻译完成后触发 transcript reflow，避免用户界面停留在旧渲染。

## 协议处理

- 支持 SSE `data.type`，并优先于普通 `event:` 字段，兼容 `event: message` + JSON `type=commit` 的形态。
- `delta`/`revise` 在 streaming 模式下只作为 partial，不提前完成；`commit`/`passthrough` 才完成显示。
- `/translate/stream` 逐 chunk 读取，遇到完整 `commit`/`passthrough`/`error` 可立即返回，不等待长连接关闭。
- `source_hidden` 和 `stats` 不进入用户可见文本；翻译失败回退到原文。

## 测试与验证

- `just fmt`：通过。
- `just test -p codex-tui hypo_translate`：8 passed。
- `just test -p codex-tui translated_history_cell`：2 passed。
- `just test -p codex-tui`：M4 主验证通过。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。

## Worker 证据

- test worker：`019e60b4-ac18-7d83-ab78-8362428ada56`，因无生产 `hypo_translate` API 暂时无法写出编译测试，报告最小 API 需求后关闭。
- implement worker：`019e60b7-59d7-7fc1-8378-be1cb836bfb6`，新增 runtime translator 与 TUI 接线。
- test follow-up：`019e60cb-1f15-77b3-b408-65a2c32d9111`，补充 fake protocol tests，发现 SSE `data.type` 解析缺口。
- implement follow-up：`019e60cf-d55a-75f2-9317-157d5fa1a7e0`，修复 `data.type` 与 transcript reflow。
- audit worker：`019e60d6-ac42-7be1-b568-960c7a0c95fe`，发现长连接 SSE 和 `event: message` 两个问题。
- follow-up workers：`019e60da-*`、`019e60df-0ebb-*`，补齐长连接、`data.type`、`delta` partial 语义红测；主代理完成最后的 `delta` pending 修复。
- final audit follow-up：`019e60fa-961a-7153-9a6f-9aa68c55aec4`，结论 pass。

## 已知说明

- 如果 SSE 连接正常关闭但没有 `commit`，当前实现会把累积的 `delta` 作为最终文本。这不是 streaming 模式下的提前完成，但如果 Hypo-Translate 协议未来强制要求必须有 `commit`，可以再收紧为必须 fallback。
- 本轮未做真实服务手工烟测；M6 会统一做可运行性和集成回归检查。
