# M4 Hypo-Translate 运行时翻译审核

## Verdict

pass

## Reviewed Refs

- `codex-rs/tui/src/hypo_translate.rs`
- `codex-rs/tui/src/history_cell/base.rs`
- `codex-rs/tui/src/history_cell/messages.rs`
- `codex-rs/tui/src/history_cell/mcp.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/tui/src/chatwidget/tool_lifecycle.rs`
- `codex-rs/tui/src/app_event.rs`
- `codex-rs/tui/src/app/event_dispatch.rs`
- M4 `hypo_translate` and `translated_history_cell` tests

## Findings

初始审计发现两个 P2 问题，均已修复：

- P2：`event: message` + JSON `data.type` 没有被识别。已让 JSON `data.type` 优先决定事件语义，并补充测试。
- P2：`/translate/stream` 使用完整 body 读取，长连接会等到关闭或 timeout 才解析。已切换为 chunk 读取，完整事件可提前返回。

后续补充审查通过：

- `delta` 在 streaming 解析中保持 pending，不会在 commit 前误判为完整译文。
- `source_hidden` 和 `stats` 不显示为用户可见文本。
- raw/original 模式仍委托原始 inner cell，不泄露翻译元数据。
- 翻译失败会 fallback/passthrough，不阻断 TUI。
- 成功异步翻译通过 `TranslatedHistoryCellUpdated` 触发 transcript reflow。

## Residual Risks

- EOF 时无 `commit` 的累积 `delta` 会被当作最终文本；当前视为低风险兼容行为，后续可根据 Hypo-Translate 协议再收紧。
- 真实 Hypo-Translate 服务可用/不可用手工烟测留到 M6 集成验收。

## Validation

- `just test -p codex-tui hypo_translate`：8 passed。
- `just test -p codex-tui translated_history_cell`：2 passed。
- `just test -p codex-tui`：通过。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。
