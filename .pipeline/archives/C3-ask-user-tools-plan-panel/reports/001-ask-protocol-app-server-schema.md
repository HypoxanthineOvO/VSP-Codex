# M1 Report: Ask 协议模型与 app-server experimental schema

完成时间：2026-05-26T21:32:13+08:00

## 结果

已完成。`request_user_input` 被扩展为可表达 request summary、request description、题型、逐题说明、option id、结构化答案和 secret 标记的协议模型；app-server v2 继续以 experimental surface 暴露 Ask。

## 主要变更

- `codex-rs/protocol/src/request_user_input.rs`
  - 新增 `RequestUserInputQuestionKind`。
  - 支持 `summary`、`description`、question description、option `id`。
  - `RequestUserInputAnswer` 支持 `selectedOptionIds`、`orderedOptionIds`、`freeText`、`legacyAnswers`，并保留 legacy `answers`。
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
  - app-server v2 `ToolRequestUserInput*` payload 同步结构化 Ask 字段。
- `codex-rs/app-server-protocol/src/protocol/v1.rs`
  - 初始化 capability 中加入 native Ask UI 支持，未复用 `experimentalApi`。
- `codex-rs/app-server/src/bespoke_event_handling.rs`
  - 使用统一转换路径把 core Ask event 映射到 app-server request。

## 验证

- `cd codex-rs && just write-app-server-schema --experimental`
- `cd codex-rs && just write-app-server-schema`
- `cd codex-rs && just test -p codex-app-server-protocol`

结果：通过。

## 风险

无阻塞风险。app-server Ask API 仍保持 experimental，稳定化不属于 C3。
