# M4 Hypo-Translate 运行时翻译

## Goal
接入本机 Hypo-Translate 服务，把运行时输出中的 reasoning/thinking、Subagent transcript、tool summaries/returns 翻译成中文，同时保留英文术语。默认使用本机端口服务；服务不可用时不阻塞 TUI，显示原文。

## Subworker Assignment Plan
Subworkers are authorized for this milestone. Worker separation is strict.

- `test`: 搭建 fake Hypo-Translate HTTP/SSE 测试，覆盖 `/translate/stream`、fallback `/translate`、服务不可用 passthrough、`source_hidden` 不显示不持久化、raw/original toggle。测试 worker 不实现客户端。
- `implement`: 实现翻译客户端、缓存/节流、TUI 接线和 raw/original toggle。实现 worker 不改静态汉化文案，那属于 M5。
- `audit`: 审核隐私、失败模式、性能、streaming UI 一致性，确认不会把隐藏源文本写入 session。
- Main agent: 协调测试优先，整合实现，运行 scoped tests。

## Technical Solution
默认连接 `http://127.0.0.1:8998`。优先调用 `POST /translate/stream`，请求字段包括 `text`、`source_lang: "auto"`、`target_lang: "zh-CN"`；如果 streaming 不可用，可以 fallback 到 `POST /translate`。SSE 事件处理：

- `delta`、`revise`、`commit`、`passthrough` 可用于 UI 渲染。
- `source_hidden` 只作为隐私信号，不能显示或持久化。
- `error` 触发 passthrough，不阻断 TUI。
- `stats` 可用于 debug/log，但不能污染用户主界面。

翻译范围只包括三类动态内容：

- reasoning/thinking 输出。
- Subagent transcript，包括命令和返回内容。
- tool summaries/returns。

静态 UI 汉化归 M5。

## Technical Route
1. 阅读 `/home/heyx/Hypo-Translate/docs/client-integration.md`，确认协议字段与 streaming 事件。
2. 找到 TUI 中 reasoning、Subagent transcript、tool output 的渲染入口。
3. 先写 fake server 测试，模拟 stream commit、error、passthrough 和 source_hidden。
4. 新增独立 translation client/module，配置默认 base URL，支持超时、失败降级和简单缓存。
5. 在 TUI 渲染数据层接入翻译结果，避免在最终字符串渲染后再做不可控替换。
6. 增加 raw/original toggle，并确保 toggle 不需要重新请求即可查看原文。

## Research Required
需要本地阅读 Hypo-Translate 文档和 Codex TUI 数据流，不需要联网。已知关键文档：

- `/home/heyx/Hypo-Translate/docs/client-integration.md`
- `/home/heyx/Hypo-Translate/docs/systemd-deployment.md`

## Risks And Alternatives
- 风险：逐 token 翻译造成 UI 卡顿。替代方案是异步翻译完整段落，先显示原文/占位，commit 后替换。
- 风险：翻译服务不可用导致主流程失败。必须 passthrough 原文并记录非阻断状态。
- 风险：隐藏源文本被写入 session。必须区分显示文本、原始事件和可持久化内容。
- 风险：术语被翻坏。替代方案是依赖 Hypo-Translate glossary，同时在 prompt 中要求保留英文术语。

## Validation Path
- `just fmt` from `codex-rs`.
- Run scoped tests for any new translation/client crate or TUI module.
- `just test -p codex-tui` if TUI rendering or snapshots change.
- Manual smoke with Hypo-Translate service running on `127.0.0.1:8998`, then stop service and confirm TUI falls back to original output.

## Audit Focus
- Translation failure never blocks user work.
- `source_hidden` content is not displayed or persisted.
- Only the three approved dynamic content categories are translated.
- Toggle between translated/raw content is understandable and keyboard accessible.

## Deliverables
- Translation client/module with fake server tests.
- TUI integration for approved dynamic content.
- Milestone report with available/unavailable service smoke evidence.
